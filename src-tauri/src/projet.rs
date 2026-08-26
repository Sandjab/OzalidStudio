//! Le projet : un livre entier dans un fichier `.ozalid`.
//!
//! L'archive est auto-portante — on l'ouvre, on la déplace, on la sauvegarde comme un
//! document, et elle reste complète sur une autre machine :
//!
//! ```text
//! projet.toml     identité du livre, réglages de couverture, chemin source du manuscrit
//! manuscrit.md
//! images/         photos source de la 1ère et de la 4ème
//! polices/        la police personnelle de l'auteur, quand il en fournit une
//! envois/         les images des envois, une par dédicataire
//! ```
//!
//! Les images d'envoi sont sous `envois/`, et **pas** avec celles de la couverture :
//! `package::ecrire_images` donne un rôle aux images du projet par leur seul nom, et
//! tout ce qui ne commence pas par `quatrieme` y devient la première de couverture. Une
//! image d'envoi versée dans ce tas remplacerait la couverture, en silence.
//!
//! `projet.toml` garde la forme et l'esprit du `livre.toml` historique : dézippée,
//! l'archive reste lisible et diffable.
//!
//! Les **sorties n'y sont pas**. Un `.ozalid` ne contient que des sources ; les
//! packages sont écrits à côté. L'archive reste légère, et aucune sortie périmée ne
//! survit à un déplacement du projet.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// La version du format `.ozalid`.
///
/// **v5** : le destinataire devient un **livrable** — POD, format, reliure, papier —, et
/// la mesure quitte le destinataire pour une map rangée par gabarit d'intérieur. Deux
/// champs se déplacent, et la clé plate d'hier ne nomme plus rien : aucune structure de
/// la v5 ne sait lire un `[[livraison.destinataires]]`, et un binaire v4 ouvrant un
/// projet v5 se retrouverait sans aucun destinataire — donc sans format de couverture.
/// Monter la version fait refuser ce projet, ce qui est vrai et réparable.
///
/// Version 4 : la main de l'envoi descend du livre dans l'exemplaire, et le gabarit de
/// diffusion remonte sur `Envois`. Contrairement aux sections facultatives ajoutées
/// depuis la v3, qu'un binaire d'avant traverse sans dommage, un champ se **déplace**
/// ici : un binaire v3 ouvrant un projet v4 ne verrait aucune main d'envoi, et son
/// `serde(default)` les lui donnerait toutes dans la même écriture — celle que
/// personne n'a choisie. Monter la version fait refuser ce projet, ce qui est vrai et
/// réparable, plutôt qu'imprimer vingt exemplaires dans la mauvaise main.
///
/// Version 3 : l'éditeur, le monogramme, la collection, le prix et la mention sont au
/// livre, là où la 2 les rangeait dans la maquette. Le livre dit ce qui est écrit, la
/// maquette dit où et si ça se voit.
///
/// Version 2 : la maquette de couverture y est typée, dans le vocabulaire du moteur
/// Typst, là où la 1 conservait le bloc de réglages brut de l'atelier HTML.
pub const VERSION: u32 = 5;
const PROJET_TOML: &str = "projet.toml";
const MANUSCRIT_MD: &str = "manuscrit.md";
const IMAGES: &str = "images/";
const POLICES: &str = "polices/";
const ENVOIS: &str = "envois/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Livre {
    pub titre: String,
    /// Titre de la page de titre, avec ses sauts de ligne voulus.
    ///
    /// Vaut `%TITRE%` par défaut, ce qui reproduit l'ancien repli — le titre sert — en
    /// le rendant visible dans le champ et retouchable. Un `.ozalid` écrit avant le
    /// jeton reçoit ce défaut ; `VERSION` n'a donc pas à bouger.
    #[serde(default = "titre_page_defaut")]
    pub titre_page: String,
    pub auteur: String,
    #[serde(default = "genre_defaut")]
    pub genre: String,
    /// L'éditeur, la collection et le monogramme : des **clés**, littérales, jamais
    /// substituées. Elles nomment la maison, pas le livre, et elles ne bougent pas d'un
    /// titre à l'autre chez un auto-éditeur.
    ///
    /// Elles vivaient dans la maquette — l'éditeur dans le pied de la 1ère, que le dos
    /// relisait ; la collection sous le nom de « pastille ». Le livre dit ce qui est
    /// écrit, la maquette dit où et si ça se voit.
    #[serde(default = "editeur_defaut")]
    pub editeur: String,
    #[serde(default = "collection_defaut")]
    pub collection: String,
    #[serde(default = "monogramme_defaut")]
    pub monogramme: String,
    #[serde(default)]
    pub copyright: String,
    /// Le prix et la mention légale : des champs **libres**, qui citent les clés.
    ///
    /// Ils naissent vides — le pied de la 4ème saute les lignes vides —, mais gardent
    /// leur générique en défaut de lecture : un `.ozalid` d'avant la v3 tenait ces deux
    /// textes dans la maquette, et les lui rendre vides effacerait un prix écrit.
    #[serde(default = "prix_defaut")]
    pub prix: String,
    #[serde(default = "mention_defaut")]
    pub mention: String,
    /// Dédicace imprimée, en belle page après le copyright. Vide, aucune page n'est
    /// composée : c'est `dedicace()` qui en juge, pas ses appelants.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub dedicace: String,
    /// Contrôle d'intégrité facultatif : il n'a de sens qu'au gel du manuscrit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapitres: Option<u32>,
}

pub(crate) fn genre_defaut() -> String {
    "Genre".into()
}

pub(crate) fn titre_page_defaut() -> String {
    "%TITRE%".into()
}

/// L'année en cours, pour dater le copyright d'un projet neuf.
///
/// Prise sur `epub::horodatage`, dont le format `AAAA-MM-JJT…` est stable et entièrement
/// ASCII : le projet n'a aucune dépendance de date, il porte son propre calendrier —
/// l'algorithme de Hinnant dans `epub::civil` — et en avoir deux implantations serait
/// pire que ce découpage.
fn annee_courante() -> String {
    crate::epub::horodatage(std::time::SystemTime::now())[..4].to_string()
}

/// Le copyright d'un projet neuf : l'auteur cité, l'année **écrite**.
///
/// L'année est figée à la création et non citée par un jeton : un `%ANNEE%` résolu à
/// chaque composition ferait dire 2028 au copyright d'un livre déposé en 2026, et le
/// dépôt légal ne se rattrape pas.
fn copyright_defaut() -> String {
    format!(
        "© %AUTEUR%, {}.\nTous droits réservés.\nMaquette de couverture : atelier Ozalid",
        annee_courante()
    )
}

fn editeur_defaut() -> String {
    "Editeur".into()
}

fn collection_defaut() -> String {
    "Collection".into()
}

fn monogramme_defaut() -> String {
    "Monogramme".into()
}

fn prix_defaut() -> String {
    "Prix".into()
}

fn mention_defaut() -> String {
    "Mention".into()
}

impl Livre {
    /// Un livre à remplir : tous les champs vides, sauf le genre, dont le défaut
    /// vaut mieux qu'un blanc — et c'est le même défaut que celui d'un `projet.toml`
    /// qui ne le porte pas.
    ///
    /// Le prix, la mention et la dédicace font exception dans l'autre sens : eux
    /// naissent vides, parce qu'un générique y composerait une ligne que personne n'a
    /// choisie. Voir `le_prix_la_mention_et_la_dedicace_naissent_vides`.
    pub fn vide() -> Self {
        Self {
            titre: "Titre".into(),
            titre_page: titre_page_defaut(),
            auteur: "Auteur".into(),
            genre: genre_defaut(),
            editeur: editeur_defaut(),
            collection: collection_defaut(),
            monogramme: monogramme_defaut(),
            copyright: copyright_defaut(),
            prix: String::new(),
            mention: String::new(),
            dedicace: String::new(),
            chapitres: None,
        }
    }

    /// Titre tel qu'il doit paraître sur la page de titre, jetons résolus.
    pub fn titre_page(&self) -> String {
        crate::gabarit::substituer(&self.titre_page, self)
    }

    /// Le copyright, jetons résolus.
    pub fn copyright(&self) -> String {
        crate::gabarit::substituer(&self.copyright, self)
    }

    /// Le prix, jetons résolus.
    pub fn prix(&self) -> String {
        crate::gabarit::substituer(&self.prix, self)
    }

    /// La mention légale, jetons résolus.
    pub fn mention(&self) -> String {
        crate::gabarit::substituer(&self.mention, self)
    }

    /// La dédicace, jetons résolus, si elle n'est pas que du blanc.
    ///
    /// Le rognage est ici et nulle part ailleurs : une dédicace réduite à une espace
    /// ajouterait sinon deux pages au livre, donc du dos, sans que rien ne se voie à
    /// l'écran. Il vient **après** la substitution, pour qu'un jeton dont la clé est
    /// vide ne compose pas davantage.
    pub fn dedicace(&self) -> Option<String> {
        let d = crate::gabarit::substituer(&self.dedicace, self);
        let d = d.trim();
        (!d.is_empty()).then(|| d.to_string())
    }
}

/// Le manuscrit est **embarqué** dans l'archive : c'est ce qui rend le `.ozalid`
/// auto-portant. Son chemin d'origine est mémorisé pour que « Réimporter le
/// manuscrit » soit un bouton, et non une navigation dans un sélecteur de fichiers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manuscrit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// La maquette de couverture du projet, dans le vocabulaire du moteur Typst.
///
/// Absente tant qu'aucune maquette n'a été choisie ni importée : composer la
/// couverture n'a alors pas d'objet, et l'interface le dit plutôt que d'en inventer
/// une par défaut.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Couverture {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maquette: Option<crate::couverture::Couverture>,
}

/// Un livrable du livre : la fabrication qu'on déclare — POD, format, reliure, papier —
/// la finition qui paraîtra au récapitulatif sans changer le fichier, et, pour les POD
/// qui ne publient ni dos ni fond perdu, ce qui a été relevé sur leur gabarit.
///
/// Les relevés naissent absents, jamais préremplis : une valeur inventée qui ressemble
/// à une mesure est pire qu'un champ vide, et le refus de composer dit quoi faire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Livrable {
    #[serde(flatten)]
    pub fabrication: crate::catalogue::Fabrication,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dos_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fond_perdu_mm: Option<f64>,
}

impl Livrable {
    /// La clé du livrable — quatre axes, l'identité entière (spec § 4).
    pub fn cle(&self) -> String {
        self.fabrication.cle()
    }

    /// Un livrable neuf sur cette fabrication : aucun relevé, aucune finition.
    pub fn pour(f: crate::catalogue::Fabrication) -> Self {
        Self {
            fabrication: f,
            finition: None,
            dos_mm: None,
            fond_perdu_mm: None,
        }
    }
}

/// Ce qu'une composition mesure, et que le projet retient — par **gabarit d'intérieur**
/// (POD, format, reliure), jamais par livrable : la pagination ne dépend ni du papier ni
/// de la finition, et c'est ce partage qui rend la comparaison de deux papiers gratuite
/// (spec § 5). Le dos n'y est plus : il dépend du papier, il se **recalcule** à chaque
/// vue depuis la formule du papier du livrable qu'on regarde.
///
/// Retenue dans le `.ozalid` et non dans une variable de l'interface : le même livre a
/// autant de paginations que de gabarits, et les redemander une à une à chaque
/// changement de lunette faisait payer une composition entière pour un chiffre déjà
/// connu — puis une deuxième fois à la réouverture. Plus `Copy` depuis qu'elle porte
/// les polices de repli : un `Vec` ne se copie pas. C'est le seul prix du champ, et il
/// se paie en `.clone()` aux rares endroits qui lisent une mesure derrière une
/// référence.
///
/// **Une mesure présente vaut toujours.** L'invariant tient, sur une clé plus large :
/// rien n'est à comparer avant usage, et ce qui pourrait la périmer l'efface à la
/// source — ou à l'ouverture, pour la seule cause qui échappe aux mutateurs : un
/// gabarit réécrit dans `<config>/pods/` pendant que le livre était fermé. C'est ce que
/// l'`empreinte` attrape (spec § 8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mesure {
    pub pages: u32,
    pub gouttiere: f64,
    pub blanche: bool,
    /// L'empreinte du gabarit qui a composé (`Resolu::empreinte`). Comparée une seule
    /// fois, à l'ouverture, par `normalise`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empreinte: Option<String>,
    /// Familles que Typst n'a pas trouvées et a remplacées par une écriture de repli.
    ///
    /// Retenues **avec la mesure** et non dans une variable de l'écran : un PDF composé
    /// dans une écriture de repli ne redevient pas juste en rouvrant le livre. Un pied
    /// qui se tairait à la réouverture dirait que tout va bien devant un fichier qui ne
    /// suit pas la maquette — et Typst, lui, n'échoue pas : il substitue et poursuit.
    ///
    /// Vide, tout va bien. Une archive écrite avant ce champ se relit vide, ce qui est
    /// exactement ce qu'elle voulait dire : `VERSION` n'a donc pas à bouger.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub polices_introuvables: Vec<String>,
}

/// À qui le livre est destiné, et pour lequel de ces livrables on regarde.
///
/// Une seule liste et un pointeur dessus : l'intérieur se compose pour le courant, la
/// couverture s'aperçoit à son format, la génération sert toute la liste. Le livrable
/// n'est donc désigné qu'une fois, là où il l'a toujours été deux.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Livraison {
    pub livrables: Vec<Livrable>,
    /// Clé du livrable visé (`Livrable::cle`) — toujours l'un des livrables ci-dessus,
    /// et une **clé**, jamais un index : un index décalé par un retrait désignerait un
    /// autre livrable en silence, et un pointeur qui ne désigne rien fait boucler la
    /// recomposition sans erreur (reconnaissance § 6). `normalise` garantit qu'elle
    /// désigne toujours quelqu'un.
    #[serde(default)]
    pub courant: String,
    /// Ce livre a été composé au moins une fois, pour n'importe lequel des
    /// destinataires. Posé à la première composition, **jamais repris** : c'est de
    /// l'histoire du projet, pas un état courant.
    ///
    /// Il dit la seule chose qu'une mesure effacée ne dit plus : la différence entre un
    /// dos qu'on n'a jamais demandé et un dos qu'une modification vient de périmer. Le
    /// premier ne réclame rien, le second réclame une recomposition.
    #[serde(default)]
    pub deja_compose: bool,
    /// Les mesures, par clé de gabarit (`Fabrication::cle_gabarit`). Une map et non une
    /// liste : deux entrées de même gabarit sont impossibles par construction.
    ///
    /// Une mesure dont plus aucun livrable ne porte le gabarit — une reliure réglée sur
    /// la ligne, depuis le lot 3 — survit jusqu'à la prochaine ouverture, où `normalise`
    /// l'élague. Personne ne la lit entre-temps : elle est rangée sous une clé que plus
    /// aucun livrable ne forme.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub mesures: std::collections::BTreeMap<String, Mesure>,
}

/// Un livre naît avec un livrable : la fabrication d'office du premier POD du catalogue
/// — c'est l'entrée que la table plate met en tête, et `Pod::fabrication_defaut` est ce
/// qui garantit que les deux désignent le même. Le pointeur ne doit jamais être vide, ne
/// serait-ce que pour regarder une première de couverture, qui réclame un format sans
/// réclamer aucune composition.
impl Default for Livraison {
    fn default() -> Self {
        let l = Livrable::pour(crate::catalogue::pods()[0].fabrication_defaut().expect(
            "tout POD lu porte format, papier et reliure composable : \
                     `Pod::verifie` les réclame",
        ));
        Self {
            courant: l.cle(),
            livrables: vec![l],
            deja_compose: false,
            mesures: std::collections::BTreeMap::new(),
        }
    }
}

impl Livraison {
    /// Le livrable visé, s'il y en a un.
    pub fn courant(&self) -> Option<&Livrable> {
        self.livrables.iter().find(|l| l.cle() == self.courant)
    }

    /// Oublie ce que toutes les compositions ont mesuré.
    ///
    /// Grossier et complet : appelé dès que quelque chose *peut* déplacer la pagination,
    /// sans regarder si elle l'a déplacée. C'est le parti déjà pris pour le manuscrit —
    /// comparer deux fois un roman entier coûte plus qu'une recomposition, et se tromper
    /// de ce côté-là n'imprime rien de faux.
    ///
    /// `deja_compose` survit : ce qui vient d'être perdu, c'est la mesure, pas le fait
    /// qu'on en ait déjà voulu une.
    pub fn oublier_mesures(&mut self) {
        self.mesures.clear();
    }

    /// Retient ce qu'une composition vient de mesurer pour un gabarit.
    ///
    /// Sans effet si plus aucun livrable ne porte ce gabarit : une composition dont le
    /// destinataire a disparu en chemin n'a personne à renseigner.
    pub fn retenir_mesure(&mut self, cle_gabarit: &str, mesure: Mesure) {
        if self
            .livrables
            .iter()
            .any(|l| l.fabrication.cle_gabarit() == cle_gabarit)
        {
            self.mesures.insert(cle_gabarit.to_string(), mesure);
            self.deja_compose = true;
        }
    }

    /// La mesure d'un gabarit, si une composition l'a faite.
    pub fn mesure(&self, cle_gabarit: &str) -> Option<&Mesure> {
        self.mesures.get(cle_gabarit)
    }

    /// Remet la liste d'accord avec le catalogue.
    ///
    /// Un `.ozalid` peut nommer un POD, un format, une reliure ou un papier que le
    /// catalogue ne porte plus, ou le même livrable deux fois. Élaguer vaut mieux que
    /// refuser d'ouvrir : le reste du projet — le manuscrit, la maquette — est intact,
    /// et la liste des livrables se refait en trois clics. C'est le même arbitrage que
    /// les projets récents dont le fichier a disparu.
    fn normalise(&mut self) {
        let mut vus = std::collections::BTreeSet::new();
        let vise = self.courant.clone();
        // Le pointeur porte le papier depuis la v5 : un repli d'office renomme le
        // livrable visé, et sans ce rattrapage `courant` ne le retrouverait plus — il
        // sauterait sur le premier de la liste, faisant rouvrir le livre sur quelqu'un
        // d'autre, en silence. La clé plate d'avant, elle, ne portait pas le papier.
        let mut rebaptise: Option<String> = None;
        self.livrables.retain_mut(|l| {
            if crate::catalogue::resout(&l.fabrication).is_err() {
                // Le papier est le seul axe qui se remplace sans changer le gabarit :
                // les trois autres partis — ou la reliure plus outillée —, le livrable
                // ne désigne plus rien de composable.
                let Some(pod) = crate::catalogue::pod(&l.fabrication.pod) else {
                    return false;
                };
                let avant = l.cle();
                l.fabrication.papier = pod.papiers[0].cle.clone();
                if crate::catalogue::resout(&l.fabrication).is_err() {
                    return false;
                }
                if avant == vise {
                    rebaptise = Some(l.cle());
                }
                // La mesure du gabarit survit : le papier ne pagine pas.
            }
            vus.insert(l.cle())
        });
        if let Some(c) = rebaptise {
            self.courant = c;
        }
        // Une mesure ne vaut que pour un gabarit encore déclaré **et** inchangé : un
        // `<config>/pods/*.toml` réécrit pendant que le livre était fermé est la seule
        // cause de péremption qui échappe aux mutateurs (spec § 8). Une mesure sans
        // empreinte — écrite avant elle — est périmée par prudence : recomposer coûte
        // des secondes, un dos affiché faux coûte une confiance.
        let gabarits: std::collections::BTreeMap<String, String> = self
            .livrables
            .iter()
            .filter_map(|l| {
                let r = crate::catalogue::resout(&l.fabrication).ok()?;
                Some((l.fabrication.cle_gabarit(), r.empreinte()))
            })
            .collect();
        self.mesures.retain(|cle, m| {
            gabarits
                .get(cle)
                .is_some_and(|e| m.empreinte.as_deref() == Some(e))
        });
        if self.livrables.is_empty() {
            *self = Self::default();
        } else if self.courant().is_none() {
            self.courant = self.livrables[0].cle();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entete {
    pub version: u32,
}

/// Ce que porte `projet.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadonnees {
    pub ozalid: Entete,
    pub livre: Livre,
    #[serde(default)]
    pub manuscrit: Manuscrit,
    #[serde(default)]
    pub couverture: Couverture,
    #[serde(default)]
    pub interieur: crate::interieur::Interieur,
    /// Facultative : un `.ozalid` écrit avant elle s'ouvre sans rien dire et se voit
    /// doté du premier gabarit de la table — ce que faisait déjà le `select` en se
    /// posant sur sa première option. `VERSION` ne bouge donc pas.
    #[serde(default)]
    pub livraison: Livraison,
    /// Facultative, comme `livraison` et la dédicace avant elle : un `.ozalid` écrit
    /// avant les envois s'ouvre sans un mot, avec une liste vide. `VERSION` ne bouge
    /// donc pas.
    #[serde(default)]
    pub envois: crate::envoi::Envois,
}

/// Un projet ouvert : les métadonnées, le texte du manuscrit, les images, les polices.
#[derive(Debug, Clone)]
pub struct Projet {
    pub meta: Metadonnees,
    pub texte: String,
    /// Nom de fichier (sans `images/`) → contenu.
    pub images: BTreeMap<String, Vec<u8>>,
    /// Nom de fichier (sans `polices/`) → contenu. Une seule police y vit à la fois :
    /// c'est celle de l'auteur, et un livre n'a qu'une main.
    pub polices: BTreeMap<String, Vec<u8>>,
    /// Nom de fichier (sans `envois/`) → contenu. Une image par envoi, désignée par
    /// `Envoi::image` : c'est ce lien-là, et non l'ordre de la liste, qui garantit
    /// qu'un exemplaire ne part pas avec le mot d'un autre.
    pub images_envois: BTreeMap<String, Vec<u8>>,
}

/// Remonte au livre les textes qu'un projet en version 2 rangeait dans la maquette.
///
/// Sur le `toml::Value` et non sur les types : en v3, `Couverture` ne porte plus ces
/// champs, il n'y a donc plus de structure Rust capable de les lire. Un projet déjà en
/// v3 traverse sans que rien ne bouge.
///
/// Un champ vide côté v2 laisse sa valeur générique : ce qui n'a jamais été saisi n'a
/// rien à remonter.
///
/// Rien n'est **retiré** de la maquette, et ce n'est pas un oubli : une fois
/// `Couverture` allégée, ces champs y sont inconnus, serde les ignore, et aucune
/// réécriture ne les conserve. Le résultat est celui visé, sans code de suppression.
fn migre(mut v: toml::Value) -> Result<Metadonnees, String> {
    let version = version_de(&v);
    if version < VERSION as i64 {
        // La collection explicite gagne ; la pastille, qui portait un nom de collection
        // sous un autre nom, ne sert que de repli.
        let repris: [(&str, &[&str], &[&str]); 5] = [
            ("editeur", &["pied", "editeur"], &[]),
            ("monogramme", &["pied", "monogramme"], &[]),
            (
                "collection",
                &["quatrieme", "collection"],
                &["pastille", "texte"],
            ),
            ("prix", &["quatrieme", "prix"], &[]),
            ("mention", &["quatrieme", "mention"], &[]),
        ];
        for (vers, depuis, repli) in repris {
            let valeur = maquette_texte(&v, depuis).or_else(|| maquette_texte(&v, repli));
            if let (Some(valeur), Some(livre)) = (
                valeur,
                v.get_mut("livre").and_then(toml::Value::as_table_mut),
            ) {
                livre.insert(vers.to_string(), toml::Value::String(valeur));
            }
        }
        // v3 → v4 : la main du livre descend dans chaque envoi, le gabarit remonte sur
        // les envois. Sur le `toml::Value` et non sur les types, pour la raison déjà
        // dite plus haut : en v4, `Envois` ne porte plus de `main`, il n'y a donc plus
        // de structure Rust capable de la lire.
        //
        // Rien n'est **retiré**, et ce n'est pas un oubli : une fois `Envois` allégée,
        // `main` y est inconnue, serde l'ignore, et aucune réécriture ne la conserve.
        if let Some(envois) = v.get_mut("envois").and_then(toml::Value::as_table_mut) {
            let ancienne = envois.get("main").cloned();
            if let Some(g) = ancienne
                .as_ref()
                .and_then(|m| m.get("gabarit"))
                .and_then(toml::Value::as_str)
            {
                envois
                    .entry("gabarit".to_string())
                    .or_insert_with(|| toml::Value::String(g.to_string()));
            }
            if let (Some(ancienne), Some(liste)) = (
                ancienne,
                envois.get_mut("liste").and_then(toml::Value::as_array_mut),
            ) {
                for e in liste {
                    // Un envoi qui porte déjà sa main est en v4 : une migration rejouée
                    // n'a pas à écraser le travail fait.
                    if let Some(t) = e.as_table_mut() {
                        t.entry("main".to_string()).or_insert(ancienne.clone());
                    }
                }
            }
        }
        migre_livraison(&mut v);
        if let Some(o) = v.get_mut("ozalid").and_then(toml::Value::as_table_mut) {
            o.insert("version".into(), toml::Value::Integer(VERSION as i64));
        }
    }
    v.try_into().map_err(|e| format!("{PROJET_TOML} : {e}"))
}

/// v4 → v5 : le destinataire devient un livrable, la mesure quitte le destinataire pour
/// la map des gabarits, `courant` devient une clé à quatre axes. La table `HERITEES`
/// donne le triplet de chaque clé plate ; un prestataire disparu est élagué, comme
/// `normalise` l'aurait fait. Rejouée sur son propre résultat, elle ne bouge rien :
/// `destinataires` absent, elle sort au premier pas.
///
/// Sur le `toml::Value` et non sur les types, comme les deux migrations qui la
/// précèdent, et pour la même raison : en v5, plus aucune structure Rust ne sait lire un
/// `destinataire`.
fn migre_livraison(v: &mut toml::Value) {
    let Some(l) = v.get_mut("livraison").and_then(toml::Value::as_table_mut) else {
        return;
    };
    let Some(anciens) = l.remove("destinataires") else {
        return; // déjà en v5, ou jamais de livraison écrite
    };
    // Les sorties anticipées d'abord : une `destinataires` qui n'est pas un tableau fait
    // renoncer, et renoncer après avoir retiré `courant` emporterait le pointeur d'un
    // fichier qu'on n'a pas su migrer.
    let Some(anciens) = anciens.as_array() else {
        return;
    };
    let ancien_courant = l
        .get("courant")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    l.remove("courant");
    let mut livrables = toml::value::Array::new();
    let mut mesures = toml::value::Table::new();
    let mut courant: Option<String> = None;
    for d in anciens {
        let Some(t) = d.as_table() else { continue };
        let Some(plate) = t.get("provider").and_then(toml::Value::as_str) else {
            continue;
        };
        let Some((_, pod, format, reliure)) = crate::catalogue::HERITEES
            .iter()
            .find(|(h, ..)| *h == plate)
        else {
            continue; // prestataire disparu : élagué
        };
        let papier = t
            .get("papier")
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let mut n = toml::value::Table::new();
        n.insert("pod".into(), (*pod).into());
        n.insert("format".into(), (*format).into());
        n.insert("reliure".into(), (*reliure).into());
        n.insert("papier".into(), papier.clone().into());
        for champ in ["dos_mm", "fond_perdu_mm"] {
            if let Some(x) = t.get(champ) {
                n.insert(champ.into(), x.clone());
            }
        }
        // La mesure quitte le destinataire. Son ancien champ `dos` voyage avec elle et
        // meurt à la désérialisation typée — serde l'ignore, rien ne le réécrit. Sans
        // empreinte, `normalise` la périmera : une recomposition, pas une perte.
        if let Some(m) = t.get("compose") {
            mesures
                .entry(format!("{pod}-{format}-{reliure}"))
                .or_insert_with(|| m.clone());
        }
        if ancien_courant.as_deref() == Some(plate) {
            courant = Some(format!("{pod}-{format}-{reliure}-{papier}"));
        }
        livrables.push(toml::Value::Table(n));
    }
    l.insert("livrables".into(), toml::Value::Array(livrables));
    if !mesures.is_empty() {
        l.insert("mesures".into(), toml::Value::Table(mesures));
    }
    if let Some(c) = courant {
        l.insert("courant".into(), toml::Value::String(c));
    }
    // `courant` absent ou orphelin : `normalise` le posera sur le premier livrable.
}

/// La version annoncée par le fichier, ou 0 s'il n'en porte pas.
fn version_de(v: &toml::Value) -> i64 {
    v.get("ozalid")
        .and_then(|o| o.get("version"))
        .and_then(toml::Value::as_integer)
        .unwrap_or(0)
}

/// La chaîne non vide rangée sous `couverture.maquette.<chemin>`, si elle y est.
fn maquette_texte(v: &toml::Value, chemin: &[&str]) -> Option<String> {
    if chemin.is_empty() {
        return None;
    }
    let mut courant = v.get("couverture")?.get("maquette")?;
    for cle in chemin {
        courant = courant.get(cle)?;
    }
    courant
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

impl Projet {
    pub fn nouveau(livre: Livre, texte: String) -> Self {
        Self {
            meta: Metadonnees {
                ozalid: Entete { version: VERSION },
                livre,
                manuscrit: Manuscrit::default(),
                couverture: Couverture::default(),
                interieur: crate::interieur::Interieur::default(),
                livraison: Livraison::default(),
                envois: crate::envoi::Envois::default(),
            },
            texte,
            images: BTreeMap::new(),
            polices: BTreeMap::new(),
            images_envois: BTreeMap::new(),
        }
    }

    /// Remplace l'identité du livre, et oublie ce que les compositions avaient mesuré.
    ///
    /// Les pages liminaires composent : une dédicace prend une belle page et sa blanche,
    /// un pavé de copyright plus long peut refluer. Le dos suit, sans que le gabarit, le
    /// papier ni la police aient bougé — c'est la cause qui échappait à tout le monde.
    ///
    /// Les deux gestes sont ici et non chez l'appelant pour qu'on ne puisse pas faire le
    /// premier en oubliant le second.
    pub fn modifier_livre(&mut self, livre: Livre) {
        self.meta.livre = livre;
        self.meta.livraison.oublier_mesures();
    }

    /// Remplace les réglages d'intérieur, et oublie les mesures : la police repagine.
    pub fn modifier_interieur(&mut self, interieur: crate::interieur::Interieur) {
        self.meta.interieur = interieur;
        self.meta.livraison.oublier_mesures();
    }

    /// Remplace le texte du manuscrit, et oublie les mesures : le texte fait la
    /// pagination, et c'est la seule cause qui ne se lise nulle part dans l'interface.
    pub fn remplacer_texte(&mut self, texte: String) {
        self.texte = texte;
        self.meta.livraison.oublier_mesures();
    }

    /// Reprend la liste des envois saisie par l'interface, et jette ce qu'elle abandonne.
    ///
    /// Un envoi retiré emporte son image : sans cet élagage, l'archive garderait le mot
    /// manuscrit d'une personne à qui l'on n'envoie plus rien, et le `.ozalid` grossirait
    /// d'un fichier que plus rien ne nomme.
    ///
    /// La police personnelle est **reposée depuis le projet** et non reprise de la
    /// saisie : c'est ce que l'archive porte, relevé dans son fichier à l'ouverture, et
    /// une saisie qui la nommerait ferait déclarer bonne une main que Typst ne
    /// trouverait pas — l'envoi partirait chez le dédicataire dans l'écriture de repli.
    /// `Envois::reprend` tenait cette garde tant que l'interface renvoyait l'objet
    /// entier ; elle la tient ici depuis que la main appartient à l'envoi.
    pub fn regler_envois(&mut self, saisie: crate::envoi::Envois) -> Result<(), String> {
        let envois = crate::envoi::Envois {
            personnelle: self.meta.envois.personnelle.clone(),
            ..saisie
        };
        envois.verifie()?;
        self.meta.envois = envois;
        self.elaguer_images_envois();
        Ok(())
    }

    /// Ne garde sous `envois/` que les images qu'un envoi nomme encore.
    fn elaguer_images_envois(&mut self) {
        let vives: Vec<String> = self
            .meta
            .envois
            .liste
            .iter()
            .filter_map(|e| e.image.clone())
            .collect();
        self.images_envois.retain(|n, _| vives.contains(n));
    }

    /// Embarque l'image d'un envoi, sous le nom que l'archive lui donnera.
    ///
    /// Le nom vient du dédicataire, pas du fichier choisi : deux photos venues du même
    /// appareil s'appellent souvent pareil, et l'une écraserait l'autre — le second
    /// exemplaire partirait avec le mot du premier. L'extension, elle, est relevée sur
    /// les octets : Typst lit le format d'une image à son extension, et un JPEG rangé
    /// sous un `.png` ne se composerait pas.
    pub fn poser_image_envoi(&mut self, index: usize, octets: Vec<u8>) -> Result<(), String> {
        let ext = crate::image::extension(&octets)
            .ok_or("image refusée : seuls le JPEG et le PNG se composent.")?;
        let e = self
            .meta
            .envois
            .liste
            .get(index)
            .ok_or("envoi introuvable : la liste a changé.")?;
        // Les images des *autres* envois : celle que celui-ci portait déjà va partir,
        // et son nom doit pouvoir resservir — sans quoi chaque essai en pousserait un
        // nouveau, « Léa-2 », « Léa-3 », pour la même personne.
        let pris: Vec<String> = self
            .meta
            .envois
            .liste
            .iter()
            .enumerate()
            .filter(|(n, _)| *n != index)
            .filter_map(|(_, e)| e.image.clone())
            .collect();
        let nom = crate::envoi::nom_image(&e.dedicataire, ext, &pris);
        self.meta.envois.liste[index].image = Some(nom.clone());
        // Estimé sur l'image reçue, et non posé à des valeurs de maison : deux photos
        // n'ont ni le même papier ni le même éclairage. Une image que le décodeur ne
        // sait pas lire n'empêche pas de la poser — Typst la lira peut-être — et elle
        // se compose alors sans détourage.
        let mut d = crate::detourage::estime(&octets).ok();
        // Une encre nommée par un code dit sa luminance, et donc exactement où poser le
        // point d'encre. Sans cela, le seuil relevé sur les pixels d'une écriture — calé
        // sur du sombre — laisserait une encre claire semi-transparente : un rose à 100
        // de luminance plafonnerait à 69 % d'opacité, le papier au travers.
        //
        // Refusée si elle n'est pas plus sombre que le papier : la rampe s'inverserait,
        // et `applique` ne saurait rien composer. L'estimation garde alors la main, ce
        // qui est toujours exécutable.
        if let (Some(d), Some(l)) = (
            d.as_mut(),
            crate::detourage::luminance_hex(&self.meta.envois.couleur),
        ) {
            if l < d.papier {
                d.encre = l;
            }
        }
        self.meta.envois.liste[index].detourage = d;
        self.images_envois.insert(nom, octets);
        // L'image que cet envoi portait avant n'est plus nommée par personne.
        self.elaguer_images_envois();
        Ok(())
    }

    /// La famille que déclare la police embarquée, relevée dans le fichier.
    ///
    /// Le `.ozalid` peut annoncer ce qu'il veut dans son TOML : c'est le fichier qui
    /// compose. Relever la famille à chaque ouverture est ce qui empêche un nom recopié
    /// à la main de désigner une police que Typst ne trouverait pas.
    pub fn police_personnelle(&self) -> Option<String> {
        self.polices
            .values()
            .next()
            .and_then(|o| crate::police::examine(o).ok())
            .map(|p| p.famille)
    }

    /// Embarque la police de l'auteur, et la donne aux envois qui s'écrivent.
    ///
    /// Une seule à la fois : la précédente s'en va, comme la photo d'une face s'en va
    /// quand on en choisit une autre. S'en servir dans le même geste est ce qu'on vient
    /// de demander — importer une écriture pour ne pas l'employer n'a pas d'usage, et
    /// l'oublier laisserait les exemplaires dans leur écriture d'avant.
    ///
    /// **Seuls les envois qui composent du texte** la reçoivent : celui qui porte une
    /// photo d'écriture ou une image générée garde sa forme. Depuis que la main
    /// appartient à l'exemplaire et non au livre, ce geste peut croiser un choix
    /// délibéré ; substituer une police à une photo effacerait le mot que l'auteur a
    /// écrit de sa main, ce qui est d'un autre ordre que remplacer une écriture par la
    /// sienne.
    pub fn poser_police(&mut self, nom: &str, octets: Vec<u8>) -> Result<(), String> {
        let famille = crate::police::examine(&octets)?.famille;
        self.polices.clear();
        self.polices.insert(nom.to_string(), octets);
        self.meta.envois.personnelle = Some(famille.clone());
        for e in &mut self.meta.envois.liste {
            if matches!(e.main, crate::envoi::Main::Police { .. }) {
                e.main = crate::envoi::Main::Police {
                    police: famille.clone(),
                };
            }
        }
        Ok(())
    }

    /// Retire la police de l'auteur, et rend aux envois qui la nommaient une main
    /// qu'on sait composer.
    ///
    /// Laisser un envoi sur une police qui vient de partir ferait refuser la
    /// composition : exact, mais inutilement — c'est le geste de l'utilisateur qui l'a
    /// retirée, et il n'a pas demandé un livre qui ne compose plus.
    ///
    /// **Ceux-là seuls** sont ramenés au défaut : un exemplaire écrit en Caveat n'a
    /// aucune raison de changer d'écriture parce qu'un autre perd la sienne.
    pub fn retirer_police(&mut self) {
        self.polices.clear();
        let Some(partie) = self.meta.envois.personnelle.take() else {
            return;
        };
        for e in &mut self.meta.envois.liste {
            if matches!(&e.main, crate::envoi::Main::Police { police } if *police == partie) {
                e.main = crate::envoi::Main::default();
            }
        }
    }

    pub fn ouvrir(chemin: &Path) -> Result<Self, String> {
        let f = File::open(chemin).map_err(|e| format!("{} : {e}", chemin.display()))?;
        Self::lire(f).map_err(|e| format!("{} : {e}", chemin.display()))
    }

    pub fn enregistrer(&self, chemin: &Path) -> Result<(), String> {
        let f = File::create(chemin).map_err(|e| format!("{} : {e}", chemin.display()))?;
        self.ecrire(f)
            .map_err(|e| format!("{} : {e}", chemin.display()))
    }

    fn lire<R: Read + Seek>(source: R) -> Result<Self, String> {
        let mut zip = ZipArchive::new(source).map_err(|e| format!("archive illisible : {e}"))?;

        let toml_brut = fichier(&mut zip, PROJET_TOML)?.ok_or_else(|| {
            format!("archive sans {PROJET_TOML} : ce n'est pas un projet Ozalid.")
        })?;
        let toml_brut = String::from_utf8(toml_brut)
            .map_err(|_| format!("{PROJET_TOML} n'est pas de l'UTF-8."))?;
        let valeur: toml::Value =
            toml::from_str(&toml_brut).map_err(|e| format!("{PROJET_TOML} : {e}"))?;
        // Le contrôle de version précède la migration, et non l'inverse : un projet venu
        // du futur doit être refusé plutôt que migré de travers.
        let version = version_de(&valeur);
        if version > VERSION as i64 {
            return Err(format!(
                "projet en version {version}, cette application lit jusqu'à la {VERSION}."
            ));
        }
        let mut meta: Metadonnees = migre(valeur)?;
        meta.livraison.normalise();

        let texte = fichier(&mut zip, MANUSCRIT_MD)?
            .ok_or_else(|| format!("archive sans {MANUSCRIT_MD}."))?;
        let texte = String::from_utf8(texte)
            .map_err(|_| format!("{MANUSCRIT_MD} n'est pas de l'UTF-8."))?;

        let mut images = BTreeMap::new();
        let mut polices = BTreeMap::new();
        let mut images_envois = BTreeMap::new();
        let noms: Vec<String> = zip.file_names().map(str::to_owned).collect();
        for nom in noms {
            for (prefixe, cible) in [
                (IMAGES, &mut images),
                (POLICES, &mut polices),
                (ENVOIS, &mut images_envois),
            ] {
                let Some(court) = nom.strip_prefix(prefixe) else {
                    continue;
                };
                // L'entrée du répertoire lui-même, que tout archiveur écrit : rien à
                // lire, et ce n'est pas un nom de travers.
                if court.is_empty() {
                    continue;
                }
                if !nom_simple(court) {
                    return Err(format!(
                        "archive refusée : « {nom} » n'est pas un simple nom de fichier."
                    ));
                }
                if let Some(oct) = fichier(&mut zip, &nom)? {
                    cible.insert(court.to_string(), oct);
                }
            }
        }

        let mut p = Self {
            meta,
            texte,
            images,
            polices,
            images_envois,
        };
        // La famille de la police personnelle est relevée dans le fichier embarqué, et
        // non lue dans le TOML : un `.ozalid` dont on aurait retiré la police, ou dont
        // le TOML nommerait une famille que le fichier ne déclare pas, composerait
        // sinon par repli — en silence, dans une écriture que personne n'a choisie. Le
        // nom devient alors introuvable et `Envois::verifie` refuse de composer.
        p.meta.envois.personnelle = p.police_personnelle();
        Ok(p)
    }

    fn ecrire<W: Write + Seek>(&self, sortie: W) -> Result<(), String> {
        let mut zip = ZipWriter::new(sortie);
        let texte_opts =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        // Les images sont déjà compressées (PNG, JPEG) : les dégonfler coûte du temps
        // pour un gain nul, parfois négatif.
        let brut_opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        let toml_brut = toml::to_string_pretty(&self.meta)
            .map_err(|e| format!("sérialisation de {PROJET_TOML} : {e}"))?;
        ajoute(&mut zip, PROJET_TOML, toml_brut.as_bytes(), texte_opts)?;
        ajoute(&mut zip, MANUSCRIT_MD, self.texte.as_bytes(), texte_opts)?;
        for (nom, oct) in &self.images {
            ajoute(&mut zip, &format!("{IMAGES}{nom}"), oct, brut_opts)?;
        }
        for (nom, oct) in &self.images_envois {
            ajoute(&mut zip, &format!("{ENVOIS}{nom}"), oct, brut_opts)?;
        }
        // Une police, elle, se dégonfle de moitié : elle est faite de tables et de
        // courbes, pas d'un flux déjà compressé.
        for (nom, oct) in &self.polices {
            ajoute(&mut zip, &format!("{POLICES}{nom}"), oct, texte_opts)?;
        }
        zip.finish().map_err(|e| format!("clôture : {e}"))?;
        Ok(())
    }
}

/// Ce qui suit `images/`, `polices/` ou `envois/` est-il un simple nom de fichier ?
///
/// L'application n'en écrit jamais d'autres : ces trois répertoires sont plats, et
/// leurs noms sont fabriqués — `couverture.jpg`, la police copiée, l'envoi assaini par
/// `envoi::nom_image`. Mais l'archive est un document qu'on s'échange, et rien n'oblige
/// celle qu'on ouvre à venir d'ici : `package::ecrire_images` et `ecrire_polices` en
/// font des chemins par `join`, qui suit ce qui remonte jusqu'à écrire ailleurs.
///
/// La contre-oblique est refusée avec la barre : elle sépare sous Windows, et une
/// archive écrite là-bas y arriverait par le même chemin.
///
/// `maquettes` s'en sert pour les mêmes raisons : une `.maquette` est une archive du
/// même genre, qu'on s'échange aussi. Le contrôle ne doit exister qu'une fois — deux
/// exemplaires divergeraient, et c'est le plus vieux qui laisserait passer.
pub(crate) fn nom_simple(court: &str) -> bool {
    court != "." && court != ".." && !court.contains(['/', '\\'])
}

/// Ajoute une entrée à une archive. Partagé avec `maquettes`, qui écrit le même genre
/// d'archive.
pub(crate) fn ajoute<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    nom: &str,
    contenu: &[u8],
    opts: SimpleFileOptions,
) -> Result<(), String> {
    zip.start_file(nom, opts)
        .map_err(|e| format!("{nom} : {e}"))?;
    zip.write_all(contenu).map_err(|e| format!("{nom} : {e}"))
}

/// Lit une entrée d'archive, ou rend `None` si elle n'y est pas. Partagé avec
/// `maquettes`.
pub(crate) fn fichier<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    nom: &str,
) -> Result<Option<Vec<u8>>, String> {
    match zip.by_name(nom) {
        Ok(mut f) => {
            let mut buf = Vec::with_capacity(f.size() as usize);
            f.read_to_end(&mut buf)
                .map_err(|e| format!("{nom} : {e}"))?;
            Ok(Some(buf))
        }
        Err(zip::result::ZipError::FileNotFound) => Ok(None),
        Err(e) => Err(format!("{nom} : {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Le repli d'autrefois — `titre_page` absent, le titre sert — devient un jeton. Un
    /// `.ozalid` écrit avant ce lot doit donc s'ouvrir avec `%TITRE%` et composer comme
    /// avant, sans que `VERSION` ait bougé.
    #[test]
    fn un_projet_sans_titre_de_page_recoit_le_jeton() {
        let toml = r#"
[ozalid]
version = 2

[livre]
titre = "Les Heures creuses"
auteur = "Ivan Pjig"
"#;
        let m: Metadonnees = toml::from_str(toml).expect("projet sans titre_page refusé");
        assert_eq!(m.livre.titre_page, "%TITRE%");
        assert_eq!(m.livre.titre_page(), "Les Heures creuses");
    }

    /// Un titre de page saisi à la main, avec ses sauts de ligne voulus, ne doit pas
    /// être touché par la substitution.
    #[test]
    fn un_titre_de_page_ecrit_a_la_main_est_rendu_tel_quel() {
        let mut l = Livre::vide();
        l.titre = "Les Heures creuses".into();
        l.titre_page = "Les Heures\ncreuses".into();
        assert_eq!(l.titre_page(), "Les Heures\ncreuses");
    }

    /// Une dédicace peut citer le livre. Le rognage et le filtre du blanc restent en
    /// place, et s'appliquent **après** la substitution : un jeton dont la clé est vide
    /// ne doit pas composer une page pour rien.
    #[test]
    fn une_dedicace_cite_les_cles_puis_est_rognee() {
        let mut l = Livre::vide();
        l.auteur = "Ivan Pjig".into();
        l.dedicace = "  Pour %AUTEUR%.  ".into();
        assert_eq!(l.dedicace().as_deref(), Some("Pour Ivan Pjig."));

        l.auteur = String::new();
        l.dedicace = "  %AUTEUR%  ".into();
        assert_eq!(
            l.dedicace(),
            None,
            "une clé vide ne doit pas coûter deux pages"
        );
    }

    /// Le prix et la mention sont des champs libres : ils citent les clés, comme le
    /// copyright.
    #[test]
    fn le_prix_et_la_mention_citent_les_cles() {
        let mut l = Livre::vide();
        l.collection = "Les Heures".into();
        l.prix = "18 € — %COLLECTION%".into();
        l.mention = "%EDITEUR%".into();
        l.editeur = "Ozalid".into();
        assert_eq!(l.prix(), "18 € — Les Heures");
        assert_eq!(l.mention(), "Ozalid");
    }

    /// Les cinq champs sont facultatifs dans le TOML : `VERSION` monte pour ce qui
    /// change de place, pas pour ce qui s'ajoute. Un projet qui ne les porte pas reçoit
    /// leurs valeurs génériques.
    #[test]
    fn un_projet_sans_les_cles_de_la_maison_recoit_les_generiques() {
        let toml = r#"
[ozalid]
version = 2

[livre]
titre = "Les Heures creuses"
auteur = "Ivan Pjig"
"#;
        let m: Metadonnees = toml::from_str(toml).expect("projet sans les clés refusé");
        assert_eq!(m.livre.editeur, "Editeur");
        assert_eq!(m.livre.collection, "Collection");
        assert_eq!(m.livre.monogramme, "Monogramme");
        assert_eq!(m.livre.prix, "Prix");
        assert_eq!(m.livre.mention, "Mention");
    }

    /// Un `.ozalid` en version 2 porte ses textes dans la maquette. Ils remontent au
    /// livre à l'ouverture, sans resaisie : c'est la première vraie migration du format,
    /// et cinq champs courts par projet valent le code qui les sauve.
    #[test]
    fn un_projet_v2_remonte_ses_textes_de_la_maquette() {
        let v2 = v2_avec(&[
            (&["pied", "editeur"], "OZALID"),
            (&["pied", "monogramme"], "O"),
            (&["quatrieme", "collection"], "Les Heures"),
            (&["quatrieme", "prix"], "18 €"),
            (&["quatrieme", "mention"], "Dépôt légal"),
        ]);
        let m = migre(v2).expect("migration refusée");
        assert_eq!(m.livre.editeur, "OZALID");
        assert_eq!(m.livre.monogramme, "O");
        assert_eq!(m.livre.collection, "Les Heures");
        assert_eq!(m.livre.prix, "18 €");
        assert_eq!(m.livre.mention, "Dépôt légal");
        assert_eq!(m.ozalid.version, VERSION, "la version doit suivre");
    }

    /// La pastille portait un nom de collection sous un autre nom — « folio » dans la
    /// maquette Bandeau. Elle supplée une collection vide : la laisser tomber ferait
    /// perdre la seule chose que ce champ disait.
    #[test]
    fn la_pastille_supplee_une_collection_vide() {
        let v2 = v2_avec(&[
            (&["quatrieme", "collection"], ""),
            (&["pastille", "texte"], "bandeau"),
        ]);
        assert_eq!(migre(v2).unwrap().livre.collection, "bandeau");
    }

    /// La collection explicite gagne toujours : le repli n'est qu'un repli.
    #[test]
    fn une_collection_explicite_bat_la_pastille() {
        let v2 = v2_avec(&[
            (&["quatrieme", "collection"], "Les Heures"),
            (&["pastille", "texte"], "bandeau"),
        ]);
        assert_eq!(migre(v2).unwrap().livre.collection, "Les Heures");
    }

    /// Un champ que la v2 laissait vide n'a rien à remonter : la générique reste.
    #[test]
    fn un_champ_vide_en_v2_laisse_sa_generique() {
        let v2 = v2_avec(&[(&["pied", "editeur"], "")]);
        assert_eq!(migre(v2).unwrap().livre.editeur, "Editeur");
    }

    /// Un projet déjà en v3 ne doit rien remonter : ses textes sont au livre.
    #[test]
    fn un_projet_v3_traverse_la_migration_sans_bouger() {
        let mut l = livre();
        l.editeur = "Ozalid".into();
        let p = Projet::nouveau(l, String::new());
        let v3: toml::Value = toml::from_str(&toml::to_string_pretty(&p.meta).unwrap()).unwrap();
        assert_eq!(migre(v3).unwrap().livre.editeur, "Ozalid");
    }

    /// Un `.ozalid` de la v3 porte sa main au livre. La perdre ferait composer les
    /// vingt exemplaires dans le défaut — en silence, dans une écriture que personne
    /// n'a choisie. Le TOML est écrit **littéralement** : ce sont les fichiers d'hier
    /// qu'il s'agit de relire, et les types d'hier n'existent plus pour les fabriquer.
    #[test]
    fn la_main_du_livre_v3_descend_dans_chaque_envoi() {
        let v3 = r#"
[ozalid]
version = 3
[livre]
titre = "Les Heures creuses"
auteur = "Ivan Pjig"
genre = "roman"
chapitres = 12
[envois.main]
mode = "police"
police = "Dancing Script"
[[envois.liste]]
dedicataire = "Léa"
contenu = "Pour Léa."
[[envois.liste]]
dedicataire = "Marc"
contenu = "À Marc."
"#;
        let m = migre(toml::from_str(v3).expect("TOML v3 illisible")).expect("migration refusée");
        assert_eq!(m.ozalid.version, VERSION);
        for e in &m.envois.liste {
            assert_eq!(
                e.main,
                crate::envoi::Main::Police {
                    police: "Dancing Script".into()
                },
                "{} a perdu la main du livre",
                e.dedicataire
            );
        }
    }

    /// Le gabarit vivait dans la main, il appartient désormais aux envois : le perdre
    /// obligerait à réécrire le prompt d'un livre qu'on rouvre.
    #[test]
    fn le_gabarit_v3_remonte_sur_les_envois() {
        let v3 = r#"
[ozalid]
version = 3
[livre]
titre = "T"
auteur = "A"
genre = "roman"
chapitres = 1
[envois.main]
mode = "diffusion"
gabarit = "une écriture à l'encre bleue : {envoi}"
[[envois.liste]]
dedicataire = "Léa"
"#;
        let m = migre(toml::from_str(v3).expect("TOML v3 illisible")).expect("migration refusée");
        assert_eq!(m.envois.gabarit, "une écriture à l'encre bleue : {envoi}");
        assert_eq!(m.envois.liste[0].main, crate::envoi::Main::Diffusion);
    }

    /// Un envoi qui porte déjà sa main est en v4 : la main du livre n'a rien à y
    /// écraser. Sans ce contrôle, une migration rejouée écraserait le travail fait.
    #[test]
    fn un_envoi_qui_a_deja_sa_main_ne_se_la_fait_pas_ecraser() {
        let mixte = r#"
[ozalid]
version = 3
[livre]
titre = "T"
auteur = "A"
genre = "roman"
chapitres = 1
[envois.main]
mode = "police"
police = "Dancing Script"
[[envois.liste]]
dedicataire = "Léa"
[envois.liste.main]
mode = "image"
"#;
        let m = migre(toml::from_str(mixte).expect("TOML illisible")).expect("migration refusée");
        assert_eq!(m.envois.liste[0].main, crate::envoi::Main::Image);
    }

    /// Le Candide réel, tel qu'un `.ozalid` v4 le porte : une clé plate, un papier. Il
    /// doit se rouvrir sur un livrable à quatre axes, sans que personne ne resaisisse
    /// quoi que ce soit. Le TOML est écrit **littéralement** : ce sont les fichiers
    /// d'hier qu'il s'agit de relire, et les types d'hier n'existent plus pour les
    /// fabriquer.
    #[test]
    fn un_projet_v4_migre_ses_destinataires_en_livrables() {
        let v4 = r#"
[ozalid]
version = 4
[livre]
titre = "Candide"
auteur = "Voltaire"
genre = "roman"
[livraison]
courant = "lulu"
[[livraison.destinataires]]
provider = "lulu"
papier = "standard"
"#;
        let m = migre(toml::from_str(v4).expect("TOML v4 illisible")).expect("migration refusée");
        assert_eq!(m.ozalid.version, VERSION);
        assert_eq!(m.livraison.livrables.len(), 1);
        assert_eq!(
            m.livraison.livrables[0].cle(),
            "lulu-108x175-broche-standard"
        );
        assert_eq!(m.livraison.courant, "lulu-108x175-broche-standard");
        assert!(
            m.livraison.mesures.is_empty(),
            "une mesure sortie de nulle part"
        );
    }

    /// La mesure quitte le destinataire pour la map des gabarits, les relevés restent
    /// sur le livrable qui les a faits, et un prestataire que la table ne porte plus est
    /// élagué — comme `normalise` l'aurait fait.
    ///
    /// Lu **avant** `normalise`, comme les tests v3 : la mesure migrée n'a pas
    /// d'empreinte, `normalise` la périmerait, et c'est le déplacement lui-même qu'on
    /// prouve ici.
    #[test]
    fn la_migration_deplace_la_mesure_sous_le_gabarit() {
        let v4 = r#"
[ozalid]
version = 4
[livre]
titre = "Candide"
auteur = "Voltaire"
genre = "roman"
[livraison]
courant = "prestataire-disparu"
deja_compose = true
[[livraison.destinataires]]
provider = "bod"
papier = "creme-90"
[livraison.destinataires.compose]
pages = 98
gouttiere = 14.0
blanche = false
dos = 7.21
[[livraison.destinataires]]
provider = "kdp-55x85"
papier = "creme"
dos_mm = 18.4
fond_perdu_mm = 3.0
[[livraison.destinataires]]
provider = "prestataire-disparu"
papier = "standard"
"#;
        let mut m =
            migre(toml::from_str(v4).expect("TOML v4 illisible")).expect("migration refusée");
        let cles: Vec<String> = m.livraison.livrables.iter().map(Livrable::cle).collect();
        assert_eq!(
            cles,
            ["bod-135x215-broche-creme-90", "kdp-55x85-broche-creme"],
            "le prestataire disparu n'a pas été élagué"
        );

        let mesure = m
            .livraison
            .mesure("bod-135x215-broche")
            .expect("la mesure n'est pas sous son gabarit");
        assert_eq!(mesure.pages, 98);
        assert_eq!(mesure.gouttiere, 14.0);
        assert_eq!(
            mesure.empreinte, None,
            "une empreinte inventée à la migration"
        );
        assert!(m.livraison.deja_compose);

        let kdp = &m.livraison.livrables[1];
        assert_eq!(kdp.dos_mm, Some(18.4));
        assert_eq!(kdp.fond_perdu_mm, Some(3.0));

        // Le pointeur désignait celui qui vient de disparaître : `migre` le laisse vide,
        // `normalise` le repose sur le premier livrable.
        assert_eq!(m.livraison.courant, "", "un pointeur inventé");
        m.livraison.normalise();
        assert_eq!(m.livraison.courant, "bod-135x215-broche-creme-90");
    }

    /// Rejouée sur son propre résultat, la migration ne bouge rien : `destinataires`
    /// absent, elle sort au premier pas. Sans ce contrôle, un `.ozalid` réenregistré
    /// verrait sa liste de livrables vidée à la relecture suivante.
    #[test]
    fn la_migration_rejouee_ne_bouge_rien() {
        let v4 = r#"
[ozalid]
version = 4
[livre]
titre = "Candide"
auteur = "Voltaire"
genre = "roman"
[livraison]
courant = "bod"
[[livraison.destinataires]]
provider = "bod"
papier = "creme-90"
[livraison.destinataires.compose]
pages = 98
gouttiere = 14.0
blanche = false
dos = 7.21
"#;
        let une = migre(toml::from_str(v4).expect("TOML v4 illisible")).expect("migration refusée");
        let deux = migre(toml::Value::try_from(&une).expect("v5 inécrivable"))
            .expect("migration rejouée refusée");
        assert_eq!(
            toml::to_string_pretty(&une).unwrap(),
            toml::to_string_pretty(&deux).unwrap()
        );
        assert_eq!(deux.livraison.livrables.len(), 1);
    }

    /// Le jumeau v4→v5 de `un_projet_v3_traverse_la_migration_sans_bouger` : tout ce que
    /// la v5 ne déplace pas — le livre, la maquette, l'intérieur, les envois — traverse
    /// au caractère près. Une migration qui remet à neuf un champ qu'elle ne devait pas
    /// toucher perdrait du travail sans rien dire.
    #[test]
    fn un_projet_v4_complet_traverse_la_migration() {
        let mut p = Projet::nouveau(livre(), String::new());
        p.meta.couverture.maquette = Some(crate::maquettes::fournie("bandeau"));
        p.meta.interieur = crate::interieur::Interieur {
            police: "Cardo".into(),
        };
        p.meta.envois = crate::envoi::Envois {
            gabarit: "une écriture à l'encre bleue : {envoi}".into(),
            liste: vec![crate::envoi::Envoi {
                dedicataire: "Léa".into(),
                contenu: "Pour Léa.".into(),
                ..Default::default()
            }],
            ..Default::default()
        };

        // La v4 du même projet : sa version, et sa livraison dans la forme d'hier.
        let mut v4 = toml::Value::try_from(&p.meta).expect("métadonnées inécrivables");
        let t = v4.as_table_mut().unwrap();
        t.insert("ozalid".into(), toml::toml! { version = 4 }.into());
        t.insert(
            "livraison".into(),
            toml::toml! {
                courant = "bod"
                deja_compose = true
                [[destinataires]]
                provider = "bod"
                papier = "creme-90"
            }
            .into(),
        );

        let m = migre(v4).expect("migration refusée");
        assert_eq!(m.ozalid.version, VERSION);
        let tel_quel = |a: &dyn Fn(&Metadonnees) -> String| a(&m) == a(&p.meta);
        assert!(
            tel_quel(&|x| toml::to_string_pretty(&x.livre).unwrap()),
            "le livre a bougé"
        );
        assert!(
            tel_quel(&|x| toml::to_string_pretty(&x.couverture).unwrap()),
            "la maquette a bougé"
        );
        assert!(
            tel_quel(&|x| toml::to_string_pretty(&x.interieur).unwrap()),
            "l'intérieur a bougé"
        );
        assert!(
            tel_quel(&|x| toml::to_string_pretty(&x.envois).unwrap()),
            "les envois ont bougé"
        );
        assert_eq!(m.livraison.livrables.len(), 1);
        assert!(m.livraison.deja_compose);
    }

    /// Un projet en version 2, maquette Bandeau, avec les textes posés là où la v2 les
    /// rangeait — sous `couverture.maquette`.
    fn v2_avec(textes: &[(&[&str], &str)]) -> toml::Value {
        let mut p = Projet::nouveau(livre(), String::new());
        p.meta.couverture.maquette = Some(crate::maquettes::fournie("bandeau"));
        let brut = toml::to_string_pretty(&p.meta)
            .unwrap()
            .replace(&format!("version = {VERSION}"), "version = 2");
        let mut v: toml::Value = toml::from_str(&brut).unwrap();
        for (chemin, valeur) in textes {
            let mut courant = v
                .get_mut("couverture")
                .and_then(|c| c.get_mut("maquette"))
                .unwrap();
            for cle in &chemin[..chemin.len() - 1] {
                courant = courant
                    .as_table_mut()
                    .unwrap()
                    .entry(*cle)
                    .or_insert_with(|| toml::Value::Table(Default::default()));
            }
            courant.as_table_mut().unwrap().insert(
                chemin[chemin.len() - 1].to_string(),
                toml::Value::String((*valeur).into()),
            );
        }
        v
    }

    /// Un projet neuf montre la maquette telle qu'elle est : ses champs portent de
    /// vraies valeurs, que le Rust reçoit et que la composition compose partout où la
    /// maquette les montre.
    #[test]
    fn un_livre_neuf_porte_ses_generiques() {
        let l = Livre::vide();
        assert_eq!(l.titre, "Titre");
        assert_eq!(l.auteur, "Auteur");
        assert_eq!(l.genre, "Genre");
        assert_eq!(l.editeur, "Editeur");
        assert_eq!(l.collection, "Collection");
        assert_eq!(l.monogramme, "Monogramme");
    }

    /// Trois champs naissent vides, et pour la même raison : vides, ils ne composent
    /// rien, et un générique y coûterait une ligne que personne n'a choisie.
    ///
    /// La dédicace est le seul champ sans interrupteur : `interieur.rs` lui compose une
    /// belle page et sa blanche dès qu'elle n'est pas vide — deux pages de plus sur tout
    /// projet neuf, donc un dos plus épais, que rien à l'écran n'expliquerait. Le prix et
    /// la mention s'impriment au pied de la 4ème, qui saute les lignes vides : un livre
    /// neuf n'a ni prix ni dépôt légal, et « Prix » imprimé sous le résumé se lit comme
    /// un oubli.
    #[test]
    fn le_prix_la_mention_et_la_dedicace_naissent_vides() {
        let l = Livre::vide();
        assert!(l.prix.is_empty());
        assert!(l.mention.is_empty());
        assert!(l.dedicace.is_empty());
        assert_eq!(l.dedicace(), None);
    }

    /// Le copyright cite l'auteur et porte l'année de création — figée, pas un jeton :
    /// un `%ANNEE%` résolu à chaque composition ferait dire 2028 au copyright d'un livre
    /// déposé en 2026, et le dépôt légal ne se rattrape pas.
    #[test]
    fn le_copyright_neuf_cite_l_auteur_et_date_de_cette_annee() {
        let mut l = Livre::vide();
        assert!(l.copyright.contains("%AUTEUR%"), "{}", l.copyright);
        assert!(l.copyright.contains("Tous droits réservés."));
        assert!(l.copyright.contains("atelier Ozalid"));
        // L'année est écrite, pas citée : elle ne doit pas bouger d'une composition à
        // l'autre.
        assert!(!l.copyright.contains("%ANNEE%"));
        assert_eq!(l.copyright.lines().count(), 3);

        l.auteur = "Ivan Pjig".into();
        assert!(l.copyright().starts_with("© Ivan Pjig, 2"));
        assert!(!l.copyright().contains('%'));
    }

    fn livre() -> Livre {
        Livre {
            titre: "Les Heures creuses".into(),
            titre_page: "Les Heures\ncreuses".into(),
            auteur: "Ivan Pjig".into(),
            genre: "roman".into(),
            editeur: "Editeur".into(),
            collection: "Collection".into(),
            monogramme: "Monogramme".into(),
            copyright: "© Ivan Pjig, 2026.\nTous droits réservés.".into(),
            prix: "Prix".into(),
            mention: "Mention".into(),
            dedicace: String::new(),
            chapitres: Some(64),
        }
    }

    fn aller_retour(p: &Projet) -> Projet {
        let mut buf = Vec::new();
        p.ecrire(Cursor::new(&mut buf)).unwrap();
        Projet::lire(Cursor::new(buf)).unwrap()
    }

    /// Un `.ozalid` n'est pas toujours celui qu'on a écrit : c'est un document qu'on
    /// s'échange, et une archive fabriquée nomme ses entrées comme elle veut. Ce qui
    /// suit `images/` devient un chemin chez `package::ecrire_images`, par un
    /// `dossier.join(nom)` qui suit docilement ce qui remonte — le fichier s'écrirait
    /// hors du dossier de travail, et l'enregistrement le reconduirait dans l'archive
    /// suivante. Le refus est ici, à la lecture, parce que les appelants ne peuvent
    /// pas deviner d'où le nom vient.
    #[test]
    fn une_entree_qui_remonte_hors_de_son_repertoire_est_refusee() {
        for nom in ["../../ailleurs.jpg", "sous/dossier.jpg", "..", "."] {
            let mut p = Projet::nouveau(livre(), "## 01 - Un\n\nTexte.\n".into());
            p.images.insert(nom.into(), vec![0xFF, 0xD8, 0xFF]);
            let mut buf = Vec::new();
            p.ecrire(Cursor::new(&mut buf)).unwrap();
            let err = Projet::lire(Cursor::new(buf))
                .expect_err(&format!("« {nom} » accepté comme nom d'image"));
            assert!(
                err.contains(nom),
                "{nom} : message muet sur le coupable — {err}"
            );
        }
    }

    /// La même garde ne doit pas refuser un projet ordinaire : les trois répertoires
    /// portent des noms de fichiers simples, et l'entrée du répertoire lui-même — que
    /// tout archiveur écrit — n'est pas un nom de travers.
    #[test]
    fn les_noms_de_fichiers_ordinaires_passent() {
        let mut p = Projet::nouveau(livre(), "## 01 - Un\n\nTexte.\n".into());
        p.images
            .insert("couverture.jpg".into(), vec![0xFF, 0xD8, 0xFF]);
        p.polices.insert("Ma Main.ttf".into(), vec![0x00, 0x01]);
        p.images_envois.insert("rex.png".into(), vec![0x89, 0x50]);
        let r = aller_retour(&p);
        assert_eq!(r.images.len(), 1);
        assert_eq!(r.polices.len(), 1);
        assert_eq!(r.images_envois.len(), 1);
    }

    /// Le `.ozalid` est le document de l'utilisateur : ce qui y entre doit en ressortir
    /// identique, sauts de ligne du titre et réglages de couverture compris. Une perte
    /// silencieuse ici, c'est une maquette à refaire.
    ///
    /// La police posée est volontairement `Cardo`, pas `EB Garamond` : avec le défaut,
    /// le test passerait même si la police n'était jamais écrite dans l'archive, la
    /// relecture la reconstruisant de toute façon par `#[serde(default)]`.
    #[test]
    fn un_projet_complet_survit_a_l_aller_retour() {
        let mut p = Projet::nouveau(livre(), "## 01 - Un\n\nTexte.\n".into());
        p.meta.manuscrit.source = Some("/travail/roman.md".into());
        p.meta.interieur.police = "Cardo".into();
        p.meta.livre.dedicace = "À M., qui a tenu la lampe.".into();
        p.meta.livre.collection = "collection « Ozalid »".into();
        let mut maquette = crate::maquettes::fournie("filets");
        maquette.pad_x = 16.5;
        maquette.titre.taille = 9.25;
        p.meta.couverture.maquette = Some(maquette);
        p.images
            .insert("couverture.jpg".into(), vec![0xFF, 0xD8, 0xFF]);

        let r = aller_retour(&p);
        assert_eq!(r.meta.livre.titre_page(), "Les Heures\ncreuses");
        assert_eq!(r.meta.livre.chapitres, Some(64));
        assert_eq!(r.meta.livre.copyright, p.meta.livre.copyright);
        assert_eq!(
            r.meta.manuscrit.source.as_deref(),
            Some("/travail/roman.md")
        );
        assert_eq!(r.meta.interieur.police, "Cardo");
        assert_eq!(
            r.meta.livre.dedicace().as_deref(),
            Some("À M., qui a tenu la lampe.")
        );
        assert_eq!(r.texte, p.texte);
        assert_eq!(r.images["couverture.jpg"], vec![0xFF, 0xD8, 0xFF]);

        // La maquette entière survit, y compris les valeurs affinées à la main : la
        // reperdre obligerait à refaire le réglage fin de la couverture.
        let m = r.meta.couverture.maquette.unwrap();
        assert_eq!(m.mode, crate::couverture::Mode::Typo);
        assert_eq!(m.pad_x, 16.5);
        assert_eq!(m.titre.taille, 9.25);
        assert_eq!(m.titre.casse, crate::couverture::Casse::Capitales);
        assert!(m.cadre.actif);
        assert_eq!(m.cadre.filet2_couleur, "#c00000");
        assert_eq!(r.meta.livre.collection, "collection « Ozalid »");
    }

    /// Un `.ozalid` écrit avant que la police ne soit réglable doit s'ouvrir, pas être
    /// refusé — même principe que le dos rendu réglable élément par élément.
    #[test]
    fn un_projet_sans_section_interieur_prend_la_police_par_defaut() {
        let toml = r#"
[ozalid]
version = 1

[livre]
titre = "Les Heures creuses"
auteur = "Ivan Pjig"
"#;
        let m: Metadonnees = toml::from_str(toml).expect("projet sans [interieur] refusé");
        assert_eq!(m.interieur.police, "EB Garamond");
    }

    /// Une fabrication écrite en clair : les quatre axes, dans l'ordre de la clé.
    fn fab(pod: &str, format: &str, reliure: &str, papier: &str) -> crate::catalogue::Fabrication {
        crate::catalogue::Fabrication {
            pod: pod.into(),
            format: format.into(),
            reliure: reliure.into(),
            papier: papier.into(),
        }
    }

    /// L'empreinte **réelle** du gabarit, telle que le catalogue la donne. Une mesure
    /// qui n'en porte pas — ou en porte une autre — est périmée par `normalise` ; les
    /// tests qui veulent voir une mesure survivre doivent donc écrire celle-ci.
    fn empreinte_de(f: &crate::catalogue::Fabrication) -> String {
        crate::catalogue::resout(f)
            .expect("fabrication hors catalogue")
            .empreinte()
    }

    /// Le lot 3 ajoute `[livraison]` sans monter `VERSION` : les `.ozalid` déjà écrits
    /// doivent s'ouvrir et se retrouver visés sur le premier gabarit du catalogue, comme
    /// le `select` s'y posait.
    #[test]
    fn un_projet_sans_section_livraison_prend_le_premier_gabarit() {
        let toml = r#"
[ozalid]
version = 2

[livre]
titre = "Les Heures creuses"
auteur = "Ivan Pjig"
"#;
        let mut m: Metadonnees = toml::from_str(toml).expect("projet sans [livraison] refusé");
        m.livraison.normalise();
        assert_eq!(m.livraison.courant, "lulu-108x175-broche-standard");
        assert_eq!(m.livraison.livrables.len(), 1);
        assert_eq!(
            m.livraison.livrables[0].fabrication.cle_gabarit(),
            "lulu-108x175-broche"
        );
    }

    /// La liste des livrables est du travail de l'utilisateur au même titre que la
    /// maquette : la reperdre, c'est refaire ses relevés de gabarit à la main. La
    /// finition voyage avec eux, bien qu'elle ne change pas le fichier produit : elle
    /// paraît au récapitulatif, et se ressaisir vaut se retrouver.
    #[test]
    fn la_liste_des_livrables_survit_a_l_aller_retour() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.livraison = Livraison {
            livrables: vec![
                Livrable {
                    finition: Some("mat".into()),
                    ..Livrable::pour(fab("bod", "135x215", "broche", "creme-90"))
                },
                Livrable {
                    dos_mm: Some(18.4),
                    fond_perdu_mm: Some(3.0),
                    ..Livrable::pour(fab("coollibri", "148x210", "broche", "mesure"))
                },
            ],
            courant: "coollibri-148x210-broche-mesure".into(),
            deja_compose: false,
            mesures: std::collections::BTreeMap::new(),
        };

        let r = aller_retour(&p);
        assert_eq!(r.meta.livraison.courant, "coollibri-148x210-broche-mesure");
        let d = r.meta.livraison.courant().expect("courant perdu");
        assert_eq!(d.dos_mm, Some(18.4));
        assert_eq!(d.fond_perdu_mm, Some(3.0));
        assert_eq!(
            r.meta.livraison.livrables[0].cle(),
            "bod-135x215-broche-creme-90"
        );
        assert_eq!(
            r.meta.livraison.livrables[0].finition.as_deref(),
            Some("mat"),
            "la finition ne survit pas au fichier"
        );
    }

    /// Une mesure quelconque : sa valeur n'importe jamais, seule sa présence est lue.
    /// Sans empreinte — c'est le cas d'une mesure posée en mémoire et relue tout de
    /// suite, sans passer par `normalise`.
    const MESURE: Mesure = Mesure {
        pages: 262,
        gouttiere: 25.0,
        blanche: true,
        empreinte: None,
        polices_introuvables: Vec::new(),
    };

    /// Le chiffre que l'application existe pour ne pas faire ressaisir doit survivre à
    /// la fermeture du livre. Sans ça, rouvrir un projet composé la veille redemande une
    /// composition entière pour retrouver un dos qui n'a pas bougé d'un micron.
    #[test]
    fn la_mesure_d_un_gabarit_survit_a_l_aller_retour() {
        let f = fab("bod", "135x215", "broche", "creme-90");
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.livraison.livrables = vec![Livrable::pour(f.clone())];
        p.meta.livraison.courant = f.cle();
        let mesure = Mesure {
            empreinte: Some(empreinte_de(&f)),
            ..MESURE
        };
        p.meta
            .livraison
            .retenir_mesure(&f.cle_gabarit(), mesure.clone());

        let r = aller_retour(&p);
        assert_eq!(
            r.meta.livraison.mesure("bod-135x215-broche"),
            Some(&mesure),
            "la mesure du gabarit n'a pas traversé le fichier"
        );
        assert!(
            r.meta.livraison.deja_compose,
            "l'histoire du livre est perdue"
        );
    }

    /// **Le test du repli de police.** Un PDF composé dans une écriture de repli ne
    /// redevient pas juste en refermant le livre : le pied doit le redire à la
    /// réouverture. Retenu à l'écran seulement, il se serait tu, et la fenêtre aurait
    /// annoncé que tout allait bien devant un fichier qui ne suit pas la maquette.
    #[test]
    fn le_repli_de_police_survit_a_l_aller_retour() {
        let f = fab("bod", "135x215", "broche", "creme-90");
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.livraison.livrables = vec![Livrable::pour(f.clone())];
        p.meta.livraison.courant = f.cle();
        let mesure = Mesure {
            empreinte: Some(empreinte_de(&f)),
            polices_introuvables: vec!["bodoni moda".into()],
            ..MESURE
        };
        p.meta
            .livraison
            .retenir_mesure(&f.cle_gabarit(), mesure.clone());

        let r = aller_retour(&p);
        assert_eq!(
            r.meta
                .livraison
                .mesure("bod-135x215-broche")
                .map(|m| m.polices_introuvables.clone()),
            Some(vec!["bodoni moda".to_string()])
        );
    }

    /// Une archive écrite avant ces champs se relit, et se relit **vide** — ce qui est
    /// exactement ce qu'elle voulait dire : rien n'avait été substitué, ou personne ne
    /// le savait. Le TOML est écrit littéralement, sous la clé de gabarit qui range
    /// désormais les mesures.
    #[test]
    fn une_mesure_sans_le_champ_se_relit_vide() {
        let ancien = r#"
[livraison]
livrables = []

[livraison.mesures.kdp-6x9-broche]
pages = 262
gouttiere = 25.0
blanche = true
"#;
        #[derive(Deserialize)]
        struct Seule {
            livraison: Livraison,
        }
        let s: Seule = toml::from_str(ancien).expect("une mesure d'avant ne se relit plus");
        let m = s
            .livraison
            .mesure("kdp-6x9-broche")
            .expect("mesure introuvable");
        assert!(m.polices_introuvables.is_empty());
        assert_eq!(m.empreinte, None);
        assert_eq!(m.pages, 262);
    }

    /// Et vide, il ne s'écrit pas : `skip_serializing_if`. Le front le reçoit donc
    /// absent dans le cas ordinaire, et c'est ce qui l'oblige à tolérer l'absence
    /// plutôt qu'un tableau vide — la même règle que la dédicace du livre.
    #[test]
    fn un_repli_vide_ne_s_ecrit_pas() {
        let ecrit = toml::to_string(&MESURE).expect("mesure inécrivable");
        assert!(
            !ecrit.contains("polices_introuvables"),
            "le champ vide encombre l'archive : {ecrit}"
        );
    }

    /// La mesure appartient au **gabarit**, pas au livrable : deux papiers du même POD,
    /// du même format et de la même reliure paginent identiquement, et c'est ce partage
    /// qui rend la comparaison de deux papiers gratuite. Un gabarit tiers, lui, ne voit
    /// rien — le même manuscrit ne fait pas le même nombre de pages en poche et en
    /// grand format.
    #[test]
    fn une_mesure_est_partagee_par_les_livrables_du_meme_gabarit() {
        let creme = fab("kdp", "6x9", "broche", "creme");
        let blanc = fab("kdp", "6x9", "broche", "blanc");
        let ailleurs = fab("lulu", "108x175", "broche", "standard");
        let mut l = Livraison {
            livrables: vec![
                Livrable::pour(creme.clone()),
                Livrable::pour(blanc.clone()),
                Livrable::pour(ailleurs.clone()),
            ],
            courant: creme.cle(),
            deja_compose: false,
            mesures: std::collections::BTreeMap::new(),
        };
        l.retenir_mesure(&creme.cle_gabarit(), MESURE);

        assert_eq!(l.mesure(&creme.cle_gabarit()), Some(&MESURE));
        assert_eq!(
            l.mesure(&blanc.cle_gabarit()),
            Some(&MESURE),
            "l'autre papier du même gabarit ne voit pas la mesure : elle serait \
             recomposée pour rien"
        );
        assert_eq!(
            l.mesure(&ailleurs.cle_gabarit()),
            None,
            "mesure de KDP prêtée à Lulu"
        );
    }

    /// Une composition dont le gabarit a disparu de la liste en chemin n'a personne à
    /// renseigner : la mesure ne s'écrit pas, et l'histoire du livre ne bouge pas.
    #[test]
    fn une_mesure_sans_livrable_ne_se_retient_pas() {
        let mut l = Livraison::default();
        l.retenir_mesure("kdp-6x9-broche", MESURE);
        assert_eq!(l.mesure("kdp-6x9-broche"), None);
        assert!(!l.deja_compose);
    }

    /// Les trois causes qui déplacent la pagination — le livre, la police, le texte —
    /// n'en laissent aucune debout. Chacune est câblée sur la méthode qui les efface, et
    /// non sur l'appelant : c'est ce qui rend impossible de modifier sans périmer.
    #[test]
    fn ce_qui_pagine_efface_toutes_les_mesures() {
        let neuf = || {
            let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
            let g = p
                .meta
                .livraison
                .courant()
                .unwrap()
                .fabrication
                .cle_gabarit();
            p.meta.livraison.retenir_mesure(&g, MESURE);
            p
        };
        let reste = |p: &Projet| !p.meta.livraison.mesures.is_empty();

        let mut p = neuf();
        assert!(reste(&p), "le montage du test ne retient rien");
        p.modifier_livre(livre());
        assert!(!reste(&p), "le livre n'a rien périmé");

        let mut p = neuf();
        p.modifier_interieur(crate::interieur::Interieur {
            police: "Cardo".into(),
        });
        assert!(!reste(&p), "la police n'a rien périmé");

        let mut p = neuf();
        p.remplacer_texte("## 02\n\nB.\n".into());
        assert!(!reste(&p), "le texte n'a rien périmé");
    }

    /// `deja_compose` n'est pas un état courant mais de l'histoire : il distingue un dos
    /// qu'on n'a jamais demandé — rien à faire — d'un dos qu'une modification vient de
    /// périmer, qui réclame une recomposition. L'effacer avec les mesures rendrait les
    /// deux situations indiscernables.
    #[test]
    fn perimer_une_mesure_n_efface_pas_l_histoire_du_livre() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        assert!(
            !p.meta.livraison.deja_compose,
            "un livre neuf serait composé"
        );
        let g = p
            .meta
            .livraison
            .courant()
            .unwrap()
            .fabrication
            .cle_gabarit();
        p.meta.livraison.retenir_mesure(&g, MESURE);

        p.remplacer_texte("## 02\n\nB.\n".into());
        assert!(p.meta.livraison.deja_compose);
    }

    /// Un POD retiré du catalogue, le même livrable deux fois : le projet s'ouvre quand
    /// même. Refuser ferait perdre le manuscrit et la maquette pour une liste qui se
    /// refait en trois clics.
    #[test]
    fn une_livraison_incoherente_est_elaguee_plutot_que_refusee() {
        let mut l = Livraison {
            livrables: vec![
                Livrable::pour(fab("disparu", "135x215", "broche", "creme-90")),
                Livrable::pour(fab("lulu", "108x175", "broche", "standard")),
                Livrable::pour(fab("lulu", "108x175", "broche", "standard")),
            ],
            courant: "disparu-135x215-broche-creme-90".into(),
            deja_compose: true,
            mesures: std::collections::BTreeMap::new(),
        };
        l.normalise();

        assert_eq!(l.livrables.len(), 1, "doublon ou inconnu conservé");
        assert_eq!(l.livrables[0].cle(), "lulu-108x175-broche-standard");
        assert_eq!(
            l.courant, "lulu-108x175-broche-standard",
            "le pointeur désigne un absent"
        );
    }

    /// Le papier est le seul axe qui se remplace sans changer le gabarit : un papier
    /// renommé retombe sur le premier du POD, et la mesure **reste** — elle appartient
    /// au gabarit, et le papier ne pagine pas. C'est ce qui change avec la v5 : avant,
    /// reprendre un papier d'office coûtait une recomposition.
    #[test]
    fn un_papier_disparu_retombe_sur_le_defaut_sans_perdre_la_mesure() {
        let bon = fab("kdp", "6x9", "broche", "creme");
        let mut l = Livraison {
            livrables: vec![Livrable::pour(fab(
                "kdp",
                "6x9",
                "broche",
                "nacre-introuvable",
            ))],
            courant: "kdp-6x9-broche-nacre-introuvable".into(),
            deja_compose: true,
            mesures: std::collections::BTreeMap::from([(
                "kdp-6x9-broche".to_string(),
                Mesure {
                    empreinte: Some(empreinte_de(&bon)),
                    ..MESURE
                },
            )]),
        };
        l.normalise();

        assert_eq!(l.livrables.len(), 1, "le livrable a été élagué");
        assert_eq!(
            l.livrables[0].fabrication.papier, "creme",
            "le papier n'est pas retombé sur le défaut du POD"
        );
        assert_eq!(l.courant, "kdp-6x9-broche-creme");
        assert!(
            l.mesure("kdp-6x9-broche").is_some(),
            "mesure perdue alors que le papier ne pagine pas"
        );
    }

    /// Le pointeur suit le livrable dont le papier retombe sur le défaut.
    ///
    /// Avant la v5, `courant` était la clé plate : elle ne portait pas le papier, et un
    /// repli d'office la laissait intacte. La clé à quatre axes, elle, change — et sans
    /// ce rattrapage le pointeur sauterait sur le premier livrable de la liste, faisant
    /// rouvrir le livre visé sur quelqu'un d'autre, en silence.
    #[test]
    fn le_pointeur_suit_le_livrable_dont_le_papier_retombe_sur_le_defaut() {
        let mut l = Livraison {
            livrables: vec![
                Livrable::pour(fab("lulu", "108x175", "broche", "standard")),
                Livrable::pour(fab("kdp", "6x9", "broche", "nacre-introuvable")),
            ],
            courant: "kdp-6x9-broche-nacre-introuvable".into(),
            deja_compose: false,
            mesures: std::collections::BTreeMap::new(),
        };
        l.normalise();

        assert_eq!(l.livrables.len(), 2, "un livrable a été élagué");
        assert_eq!(
            l.courant, "kdp-6x9-broche-creme",
            "le pointeur a changé de livrable au lieu de suivre le sien"
        );
    }

    /// Une mesure écrite avant l'empreinte — le cas d'un `.ozalid` migré — est périmée
    /// par prudence : recomposer coûte des secondes, un dos affiché faux coûte une
    /// confiance. `deja_compose` survit : c'est « dos périmé », pas « jamais composé ».
    #[test]
    fn une_mesure_sans_empreinte_est_perimee_a_l_ouverture() {
        let f = fab("kdp", "6x9", "broche", "creme");
        let mut l = Livraison {
            livrables: vec![Livrable::pour(f.clone())],
            courant: f.cle(),
            deja_compose: false,
            mesures: std::collections::BTreeMap::new(),
        };
        l.retenir_mesure(&f.cle_gabarit(), MESURE);
        assert!(l.deja_compose);
        l.normalise();

        assert!(
            l.mesures.is_empty(),
            "une mesure sans empreinte a survécu à l'ouverture"
        );
        assert!(l.deja_compose, "l'histoire du livre est perdue");
    }

    /// Un `<config>/pods/*.toml` réécrit pendant que le livre était fermé est la seule
    /// cause de péremption qui échappe aux mutateurs (spec § 8). L'empreinte l'attrape à
    /// l'ouverture : la mesure d'un gabarit qui n'est plus le même s'en va.
    #[test]
    fn un_gabarit_reecrit_perime_la_mesure_a_l_ouverture() {
        let f = fab("lulu", "108x175", "broche", "standard");
        let mut l = Livraison {
            livrables: vec![Livrable::pour(f.clone())],
            courant: f.cle(),
            deja_compose: true,
            mesures: std::collections::BTreeMap::from([(
                f.cle_gabarit(),
                Mesure {
                    empreinte: Some("108x175|fausse|marges|d-un|autre|fichier".into()),
                    ..MESURE
                },
            )]),
        };
        l.normalise();

        assert!(
            l.mesures.is_empty(),
            "la mesure d'un gabarit réécrit a survécu"
        );
    }

    /// Une reliure que le POD annonce sans l'outiller ne compose rien : `resout` la
    /// refuse, et le livrable qui la nomme s'élague à l'ouverture plutôt que de mener à
    /// un refus après une couverture réglée (spec § 9).
    #[test]
    fn une_reliure_non_outillee_est_elaguee_a_l_ouverture() {
        let mut l = Livraison {
            livrables: vec![
                Livrable::pour(fab("bod", "135x215", "rigide", "creme-90")),
                Livrable::pour(fab("bod", "135x215", "broche", "creme-90")),
            ],
            courant: "bod-135x215-rigide-creme-90".into(),
            deja_compose: false,
            mesures: std::collections::BTreeMap::new(),
        };
        l.normalise();

        assert_eq!(l.livrables.len(), 1, "la reliure non outillée a survécu");
        assert_eq!(l.livrables[0].cle(), "bod-135x215-broche-creme-90");
        assert_eq!(l.courant, "bod-135x215-broche-creme-90");
    }

    /// Le pointeur ne peut pas être vide : sans lui, même regarder une première de
    /// couverture est impossible, faute de format.
    #[test]
    fn une_livraison_videe_reprend_le_premier_gabarit() {
        let mut l = Livraison {
            livrables: vec![],
            courant: String::new(),
            deja_compose: false,
            mesures: std::collections::BTreeMap::new(),
        };
        l.normalise();
        assert_eq!(l.livrables.len(), 1);
        assert!(l.courant().is_some());
    }

    #[test]
    fn un_projet_sans_couverture_ni_images_reste_valide() {
        let p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        let r = aller_retour(&p);
        assert!(r.meta.couverture.maquette.is_none());
        assert!(r.images.is_empty());
    }

    /// Ouvrir autre chose qu'un projet doit le dire clairement, plutôt que d'échouer
    /// plus loin sur un manuscrit vide.
    #[test]
    fn une_archive_etrangere_est_refusee_explicitement() {
        let mut buf = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buf));
            zip.start_file("autre.txt", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"rien a voir").unwrap();
            zip.finish().unwrap();
        }
        let err = Projet::lire(Cursor::new(buf)).unwrap_err();
        assert!(err.contains("projet Ozalid"), "{err}");
    }

    #[test]
    fn un_fichier_qui_n_est_pas_une_archive_est_refuse() {
        let err = Projet::lire(Cursor::new(b"pas un zip".to_vec())).unwrap_err();
        assert!(err.contains("archive illisible"), "{err}");
    }

    /// Un projet écrit par une version future doit être refusé, pas lu de travers :
    /// un champ ignoré silencieusement se traduirait par une planche fausse.
    #[test]
    fn un_projet_plus_recent_que_l_application_est_refuse() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.ozalid.version = VERSION + 1;
        let mut buf = Vec::new();
        p.ecrire(Cursor::new(&mut buf)).unwrap();
        let err = Projet::lire(Cursor::new(buf)).unwrap_err();
        assert!(err.contains("version"), "{err}");
    }

    /// L'archive ne doit contenir que des sources : y glisser des sorties la ferait
    /// gonfler et transporter des PDF périmés.
    #[test]
    fn l_archive_ne_contient_que_des_sources() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.images.insert("quatrieme.png".into(), vec![1, 2, 3]);
        let mut buf = Vec::new();
        p.ecrire(Cursor::new(&mut buf)).unwrap();

        let zip = ZipArchive::new(Cursor::new(buf)).unwrap();
        let mut noms: Vec<&str> = zip.file_names().collect();
        noms.sort_unstable();
        assert_eq!(
            noms,
            vec!["images/quatrieme.png", "manuscrit.md", "projet.toml"]
        );
    }

    /// Un `.ozalid` écrit avant la dédicace s'ouvre sans un mot : le champ est
    /// facultatif, `VERSION` n'a donc pas bougé. Même principe que la police et que
    /// `[livraison]` avant elle.
    #[test]
    fn un_projet_sans_champ_dedicace_se_relit() {
        let toml = r#"
[ozalid]
version = 2

[livre]
titre = "Les Heures creuses"
auteur = "Ivan Pjig"
"#;
        let m: Metadonnees = toml::from_str(toml).expect("projet sans dédicace refusé");
        assert!(m.livre.dedicace.is_empty());
        assert_eq!(
            m.livre.dedicace(),
            None,
            "aucune page ne doit être composée"
        );
    }

    /// Un `.ozalid` écrit avant les envois s'ouvre sans un mot : troisième section
    /// facultative après `[interieur]` et `[livraison]`, et `VERSION` n'a pas bougé.
    #[test]
    fn un_projet_sans_section_envois_se_relit() {
        let toml = r#"
[ozalid]
version = 2

[livre]
titre = "Les Heures creuses"
auteur = "Ivan Pjig"
"#;
        let m: Metadonnees = toml::from_str(toml).expect("projet sans [envois] refusé");
        assert!(m.envois.liste.is_empty());
        assert!(
            m.envois.verifie().is_ok(),
            "la main par défaut doit être valide"
        );
    }

    /// Les envois sont du travail de l'utilisateur au même titre que la maquette : les
    /// reperdre, c'est réécrire tous les mots à la main. L'écriture choisie en fait
    /// partie depuis qu'elle appartient à l'exemplaire, et non plus au livre.
    #[test]
    fn les_envois_survivent_a_l_aller_retour() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.envois.liste = vec![crate::envoi::Envoi {
            dedicataire: "Léa".into(),
            contenu: "À Léa, qui a lu la première version.".into(),
            main: crate::envoi::Main::Police {
                police: crate::envoi::MAINS[1].into(),
            },
            ..Default::default()
        }];

        let r = aller_retour(&p);
        assert_eq!(r.meta.envois.liste.len(), 1);
        assert_eq!(r.meta.envois.liste[0].dedicataire, "Léa");
        assert_eq!(
            r.meta.envois.liste[0].contenu,
            "À Léa, qui a lu la première version."
        );
        assert_eq!(
            r.meta.envois.liste[0].main,
            crate::envoi::Main::Police {
                police: crate::envoi::MAINS[1].into()
            },
            "l'écriture choisie pour Léa n'a pas survécu"
        );
    }

    /// Une police de la maison qui n'est **pas** une main : sa famille ne figure pas
    /// dans `MAINS`. C'est ce qui compte ici — avec Caveat, dont la famille est déjà la
    /// main par défaut, aucun de ces tests ne pourrait échouer.
    const FOURNIE: &[u8] = include_bytes!("../fonts/EBGaramond[wght].ttf");
    const FAMILLE: &str = "EB Garamond";

    /// Le `.ozalid` doit être auto-portant : un projet composé avec l'écriture de son
    /// auteur doit se recomposer à l'identique sur une autre machine, où cette police
    /// n'est installée nulle part. Les octets voyagent donc dans l'archive, et la main
    /// de l'envoi qui les nomme avec.
    #[test]
    fn la_police_personnelle_voyage_dans_l_archive() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.envois.liste = vec![crate::envoi::Envoi {
            dedicataire: "Léa".into(),
            ..Default::default()
        }];
        p.poser_police("main.ttf", FOURNIE.to_vec()).unwrap();
        assert_eq!(
            p.meta.envois.liste[0].main,
            crate::envoi::Main::Police {
                police: FAMILLE.into()
            }
        );

        let r = aller_retour(&p);
        assert_eq!(r.polices["main.ttf"], FOURNIE);
        assert_eq!(r.meta.envois.personnelle.as_deref(), Some(FAMILLE));
        assert_eq!(
            r.meta.envois.liste[0].main,
            crate::envoi::Main::Police {
                police: FAMILLE.into()
            },
            "l'exemplaire de Léa a perdu l'écriture de l'auteur"
        );
        assert!(r.meta.envois.verifie().is_ok(), "main perdue à l'ouverture");
    }

    /// Ce n'est pas le TOML qui compose, c'est le fichier. Un `.ozalid` qui annonce une
    /// famille que son archive ne porte pas — police retirée, TOML recopié à la main —
    /// doit se retrouver sans police personnelle, et sa main être refusée : composer par
    /// repli enverrait au dédicataire un mot dans une écriture que personne n'a choisie.
    #[test]
    fn une_police_annoncee_mais_absente_ne_compose_pas() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.envois.personnelle = Some("Ma Main".into());
        p.meta.envois.liste = vec![crate::envoi::Envoi {
            dedicataire: "Léa".into(),
            main: crate::envoi::Main::Police {
                police: "Ma Main".into(),
            },
            ..Default::default()
        }];

        let r = aller_retour(&p);
        assert_eq!(r.meta.envois.personnelle, None);
        let err = r.meta.envois.verifie().unwrap_err();
        assert!(err.contains("Ma Main"), "{err}");
    }

    /// Une seule police à la fois : la précédente s'en va, comme la photo d'une face
    /// quand on en choisit une autre. Deux polices dans l'archive laisseraient l'ordre
    /// alphabétique décider laquelle écrit les envois.
    #[test]
    fn une_police_choisie_remplace_la_precedente() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.poser_police("premiere.ttf", FOURNIE.to_vec()).unwrap();
        p.poser_police("seconde.ttf", FOURNIE.to_vec()).unwrap();
        assert_eq!(p.polices.keys().collect::<Vec<_>>(), ["seconde.ttf"]);
    }

    /// Retirer la police rend aux envois qui la nommaient une main qu'on sait composer :
    /// les laisser sur une écriture qui vient de partir ferait refuser la composition,
    /// exactement, mais pour rien — l'utilisateur n'a pas demandé un livre qui ne
    /// compose plus.
    #[test]
    fn retirer_la_police_rend_aux_envois_une_main_composable() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.envois.liste = vec![crate::envoi::Envoi {
            dedicataire: "Léa".into(),
            ..Default::default()
        }];
        p.poser_police("main.ttf", FOURNIE.to_vec()).unwrap();
        p.retirer_police();
        assert!(p.polices.is_empty());
        assert_eq!(p.meta.envois.personnelle, None);
        assert_eq!(p.meta.envois.liste[0].main, crate::envoi::Main::default());
        assert!(
            p.meta.envois.verifie().is_ok(),
            "l'exemplaire de Léa ne compose plus"
        );
    }

    /// La main embarquée que l'auteur a choisie pour un exemplaire ne doit pas être
    /// emportée par le retrait de sa police personnelle : elle ne dépendait pas d'elle.
    #[test]
    fn retirer_la_police_ne_touche_pas_a_une_main_embarquee() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.poser_police("main.ttf", FOURNIE.to_vec()).unwrap();
        let choisie = crate::envoi::Main::Police {
            police: crate::envoi::MAINS[1].into(),
        };
        p.meta.envois.liste = vec![crate::envoi::Envoi {
            dedicataire: "Léa".into(),
            main: choisie.clone(),
            ..Default::default()
        }];
        p.retirer_police();
        assert_eq!(p.meta.envois.liste[0].main, choisie);
    }

    /// Poser sa police est un geste sur l'écriture, pas sur le mot : elle va aux
    /// exemplaires qui composent du texte, et laisse intact celui qui porte une photo
    /// d'écriture ou une image générée. Depuis que la main appartient à l'exemplaire,
    /// ce geste croise des choix délibérés — substituer une police à une photo
    /// effacerait le mot que l'auteur a écrit de sa main, ce qui ne se rattrape pas.
    #[test]
    fn une_police_posee_laisse_intact_l_envoi_qui_porte_une_image() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.envois.liste = vec![
            crate::envoi::Envoi {
                dedicataire: "Léa".into(),
                ..Default::default()
            },
            crate::envoi::Envoi {
                dedicataire: "Marc".into(),
                main: crate::envoi::Main::Image,
                ..Default::default()
            },
        ];
        p.poser_police("main.ttf", FOURNIE.to_vec()).unwrap();
        assert_eq!(
            p.meta.envois.liste[0].main,
            crate::envoi::Main::Police {
                police: FAMILLE.into()
            },
            "Léa n'a pas reçu l'écriture qu'on vient d'importer"
        );
        assert_eq!(
            p.meta.envois.liste[1].main,
            crate::envoi::Main::Image,
            "le mot que l'auteur a écrit à la main pour Marc a été remplacé"
        );
    }

    /// Le retrait ne ramène au défaut que les exemplaires qui nommaient la police
    /// partie : un envoi écrit dans une main de la maison n'a aucune raison de changer
    /// d'écriture parce qu'un autre perd la sienne.
    #[test]
    fn retirer_la_police_ne_ramene_que_les_envois_qui_la_nommaient() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.poser_police("main.ttf", FOURNIE.to_vec()).unwrap();
        let autre = crate::envoi::Main::Police {
            police: crate::envoi::MAINS[1].into(),
        };
        p.meta.envois.liste = vec![
            crate::envoi::Envoi {
                dedicataire: "Léa".into(),
                main: crate::envoi::Main::Police {
                    police: FAMILLE.into(),
                },
                ..Default::default()
            },
            crate::envoi::Envoi {
                dedicataire: "Marc".into(),
                main: autre.clone(),
                ..Default::default()
            },
        ];
        p.retirer_police();
        assert_eq!(
            p.meta.envois.liste[0].main,
            crate::envoi::Main::default(),
            "Léa reste sur une écriture qui a quitté l'archive"
        );
        assert_eq!(
            p.meta.envois.liste[1].main, autre,
            "Marc a changé d'écriture parce que Léa a perdu la sienne"
        );
    }

    /// Un manuscrit renommé en `.ttf` n'est pas une écriture : le refus est au moment du
    /// choix, seul endroit où il peut encore être corrigé, et l'archive reste intacte.
    #[test]
    fn un_fichier_qui_n_est_pas_une_police_ne_devient_pas_la_main() {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.envois.liste = vec![crate::envoi::Envoi {
            dedicataire: "Léa".into(),
            ..Default::default()
        }];
        let avant = p.meta.envois.liste[0].main.clone();
        let err = p.poser_police("faux.ttf", b"## 01 - Un\n\nTexte.".to_vec());
        assert!(err.is_err());
        assert!(p.polices.is_empty(), "l'archive a gardé le faux fichier");
        assert_eq!(p.meta.envois.personnelle, None);
        assert_eq!(p.meta.envois.liste[0].main, avant);
    }

    /// Un PNG réduit à ce que `image::dimensions` sait lire : sa signature et son IHDR.
    fn png(largeur: u32) -> Vec<u8> {
        let mut p = b"\x89PNG\r\n\x1a\n".to_vec();
        p.extend(13u32.to_be_bytes());
        p.extend(b"IHDR");
        p.extend(largeur.to_be_bytes());
        p.extend(400u32.to_be_bytes());
        p.extend([8, 6, 0, 0, 0]);
        p
    }

    /// Une image décodable, contrairement à `png()` qui n'est qu'un en-tête : la crate
    /// `image` en lit les pixels, et l'estimation des seuils en a besoin.
    fn photo() -> Vec<u8> {
        let mut img = image::RgbaImage::from_pixel(16, 16, image::Rgba([243, 241, 236, 255]));
        for x in 0..16 {
            img.put_pixel(x, 8, image::Rgba([32, 38, 120, 255]));
        }
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    fn avec_envois(qui: &[&str]) -> Projet {
        let mut p = Projet::nouveau(livre(), "## 01\n\nA.\n".into());
        p.meta.envois.liste = qui
            .iter()
            .map(|d| crate::envoi::Envoi {
                dedicataire: (*d).into(),
                main: crate::envoi::Main::Image,
                ..Default::default()
            })
            .collect();
        p
    }

    /// L'image écrite à la main est une source au même titre que le manuscrit : elle
    /// voyage dans l'archive, et le nom qu'elle y prend est celui que la source Typst
    /// écrira. Le perdre, c'est un envoi qui ne compose plus.
    #[test]
    fn une_image_d_envoi_voyage_dans_l_archive() {
        let mut p = avec_envois(&["Léa"]);
        p.poser_image_envoi(0, png(300)).unwrap();

        let r = aller_retour(&p);
        assert_eq!(r.meta.envois.liste[0].image.as_deref(), Some("Léa.png"));
        assert_eq!(r.images_envois["Léa.png"], png(300));
    }

    /// Le nom vient du dédicataire, jamais du fichier choisi : deux photos du même
    /// appareil s'appellent pareil, et la seconde écraserait la première — le second
    /// exemplaire partirait avec le mot du premier.
    #[test]
    fn deux_envois_ne_partagent_jamais_leur_image() {
        // Deux personnes du même prénom : le cas où le nom seul ne suffit pas.
        let mut p = avec_envois(&["Léa", "Léa"]);
        p.poser_image_envoi(0, png(300)).unwrap();
        p.poser_image_envoi(1, png(500)).unwrap();

        assert_eq!(p.meta.envois.liste[0].image.as_deref(), Some("Léa.png"));
        assert_eq!(p.meta.envois.liste[1].image.as_deref(), Some("Léa-2.png"));
        assert_eq!(p.images_envois["Léa.png"], png(300), "images échangées");
        assert_eq!(p.images_envois["Léa-2.png"], png(500));
    }

    /// Une image remplacée ne laisse rien derrière elle : l'archive garderait sinon des
    /// mots que plus aucun envoi ne nomme, et grossirait d'une photo par essai.
    #[test]
    fn une_image_remplacee_ne_laisse_rien_dans_l_archive() {
        let mut p = avec_envois(&["Léa"]);
        p.poser_image_envoi(0, png(300)).unwrap();
        p.poser_image_envoi(0, png(500)).unwrap();

        assert_eq!(p.images_envois.len(), 1, "l'ancienne image est restée");
        // Le nom ne dérive pas d'un essai à l'autre : c'est celui du dédicataire, et
        // l'image qu'il portait vient de partir. « Léa-2 », « Léa-3 » à chaque tentative
        // donneraient à lire une file d'attente là où il n'y a qu'une personne.
        assert_eq!(p.meta.envois.liste[0].image.as_deref(), Some("Léa.png"));
        assert_eq!(
            p.images_envois["Léa.png"],
            png(500),
            "l'image n'a pas changé"
        );
    }

    /// Un envoi retiré emporte son image : c'est le mot écrit pour quelqu'un à qui l'on
    /// n'envoie plus rien.
    #[test]
    fn un_envoi_retire_emporte_son_image() {
        let mut p = avec_envois(&["Léa", "Marie"]);
        p.poser_image_envoi(0, png(300)).unwrap();
        p.poser_image_envoi(1, png(500)).unwrap();

        let restants = crate::envoi::Envois {
            liste: vec![p.meta.envois.liste[1].clone()],
            ..p.meta.envois.clone()
        };
        p.regler_envois(restants).unwrap();

        assert_eq!(p.images_envois.keys().collect::<Vec<_>>(), ["Marie.png"]);
    }

    /// Ni PNG ni JPEG, Typst ne saurait pas la poser : le refus est au moment du choix,
    /// seul endroit où il peut encore être corrigé.
    #[test]
    fn une_image_d_envoi_illisible_est_refusee() {
        let mut p = avec_envois(&["Léa"]);
        assert!(p.poser_image_envoi(0, b"pas une image".to_vec()).is_err());
        assert!(p.images_envois.is_empty());
        assert_eq!(p.meta.envois.liste[0].image, None);
    }

    /// Les images d'envoi ne rejoignent pas celles de la couverture : là-bas, tout ce
    /// qui ne s'appelle pas `quatrieme…` **devient** la première de couverture. Un mot
    /// manuscrit versé dans ce tas remplacerait la couverture du livre, en silence.
    #[test]
    fn les_images_d_envoi_ne_se_melent_pas_a_celles_de_la_couverture() {
        let mut p = avec_envois(&["Léa"]);
        p.images
            .insert("couverture.jpg".into(), vec![0xFF, 0xD8, 0xFF]);
        p.poser_image_envoi(0, png(300)).unwrap();

        let mut buf = Vec::new();
        p.ecrire(Cursor::new(&mut buf)).unwrap();
        let zip = ZipArchive::new(Cursor::new(buf)).unwrap();
        let mut noms: Vec<&str> = zip.file_names().collect();
        noms.sort_unstable();
        assert_eq!(
            noms,
            vec![
                "envois/Léa.png",
                "images/couverture.jpg",
                "manuscrit.md",
                "projet.toml"
            ]
        );
    }

    /// Une dédicace faite d'espaces ne doit pas coûter deux pages et du dos : c'est
    /// l'accesseur qui tranche, une seule fois, pour tous ses appelants.
    #[test]
    fn une_dedicace_de_blanc_equivaut_a_pas_de_dedicace() {
        let mut l = livre();
        assert_eq!(l.dedicace(), None);
        l.dedicace = "   \n  ".into();
        assert_eq!(l.dedicace(), None, "du blanc a été pris pour une dédicace");
        l.dedicace = "  À M.  ".into();
        assert_eq!(
            l.dedicace().as_deref(),
            Some("À M."),
            "les bords doivent être rognés"
        );
    }

    /// Une photo posée après ce chantier naît détourée : c'est le cas d'usage, et
    /// demander un geste de plus pour l'obtenir reviendrait à livrer le défaut par
    /// défaut.
    #[test]
    fn une_photo_posee_nait_detouree() {
        let mut p = avec_envois(&["Léa"]);
        p.poser_image_envoi(0, photo()).unwrap();
        let d = p.meta.envois.liste[0]
            .detourage
            .expect("aucun détourage posé");
        assert!(d.papier > d.encre, "seuils incohérents : {d:?}");
    }

    /// Une encre nommée par un code pose le point d'encre à sa luminance : sans quoi le
    /// seuil relevé sur les pixels d'une écriture — calé sur du sombre — laisserait un
    /// rose à 69 % d'opacité, délavé par le papier au travers.
    #[test]
    fn une_encre_en_code_pose_son_point_d_encre() {
        let mut p = avec_envois(&["Léa"]);
        p.meta.envois.couleur = "#F532AC".into();
        p.poser_image_envoi(0, photo()).unwrap();
        let d = p.meta.envois.liste[0].detourage.expect("aucun détourage");
        assert!((d.encre - 100.3).abs() < 0.5, "point d'encre {}", d.encre);
    }

    /// Une encre **plus claire que le papier** ne pose rien : la rampe s'inverserait, et
    /// `applique` refuserait de composer. Mieux vaut l'estimation relevée sur les pixels
    /// qu'un réglage que rien ne pourrait exécuter.
    #[test]
    fn une_encre_plus_claire_que_le_papier_ne_pose_rien() {
        let mut p = avec_envois(&["Léa"]);
        p.meta.envois.couleur = "#FFFFFF".into();
        p.poser_image_envoi(0, photo()).unwrap();
        let d = p.meta.envois.liste[0].detourage.expect("aucun détourage");
        assert!(d.encre < d.papier, "seuils inexécutables : {d:?}");
    }

    /// Un mot n'est pas un code : l'estimation relevée sur l'image garde la main.
    #[test]
    fn une_encre_nommee_laisse_l_estimation_faire() {
        let mut p = avec_envois(&["Léa"]);
        p.meta.envois.couleur = "blue-black".into();
        p.poser_image_envoi(0, photo()).unwrap();
        let avec = p.meta.envois.liste[0].detourage.unwrap();
        let sans = crate::detourage::estime(&photo()).unwrap();
        assert_eq!(avec, sans);
    }
}
