//! Le catalogue des POD : ce que chaque imprimeur offre, et d'où chaque chiffre vient.
//!
//! Un fichier TOML par POD. Les fournis sont incorporés au binaire par `include_str!` —
//! il n'y a donc aucun chemin à résoudre pour eux, aucun mode dégradé, aucun écart entre
//! développement et livraison. Le poste en dépose d'autres dans `<config>/pods/`, qui
//! remplacent le fourni de même clé. Ce module porte les types, leur lecture, les six
//! fichiers fournis, le chargement et la vue plate.
//!
//! Un prestataire se décrit à un seul endroit, son fichier, où la couverture — fond
//! perdu, formule de dos — et l'intérieur — format, marges, gouttières — se tiennent
//! ensemble. Séparés, ils dérivaient : la même pagination désignait deux formats.
//!
//! **Aucune valeur n'est reconstituée.** Chacune vient d'un relevé — guide, gabarit ou
//! calculateur du prestataire, parfois un livre réel — que le `source` du bloc où elle
//! sert cite. Hors tranche connue, on refuse plutôt que d'extrapoler. Ce sont les
//! `dos_…_ancre_sur_…` du module qui tiennent ces relevés : ils ne comparent le
//! catalogue à rien d'interne, ils l'ancrent au dehors.
//!
//! Cinq axes : le POD, ses formats, ses reliures, ses finitions, ses papiers. Le cas
//! courant — tout compatible avec tout — ne s'écrit pas ; seules les exceptions se
//! déclarent. Un arbre POD > format > reliure > papier aurait obligé à recopier les
//! quatre papiers d'un POD sous chacun de ses formats.
//!
//! Règle qui tient tout le reste : **une valeur qu'on n'a pas lue ne s'écrit pas**, et
//! une valeur que le code ne sait pas appliquer est refusée plutôt qu'ignorée. Le fichier
//! de données ne doit pas pouvoir promettre plus que le code. De là les trois refus du
//! module : l'énumération fermée pour ce qui n'a qu'un jeu de valeurs licites,
//! `deny_unknown_fields` pour le champ que personne ne lira, et `verifie` pour ce que
//! serde ne sait pas dire — un nombre impossible, une clé en double, une liste vide.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// Épaisseur du dos. Trois formes, parce que les prestataires en publient trois.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(tag = "forme", rename_all = "lowercase", deny_unknown_fields)]
pub enum Dos {
    /// Lulu : `pages / par + plus` mm. Gardée sous forme de division, comme le guide
    /// l'écrit — la convertir en facteur décimal introduirait une dérive.
    Divise { par: f64, plus: f64 },
    /// BoD, KDP, TheBookEdition, Bookvault : `pages × par + plus` mm. `plus` vaut 0 chez
    /// qui ne compte pas l'épaisseur de la couverture.
    Multiplie { par: f64, plus: f64 },
    /// CoolLibri : aucune formule publiable (la « main » des papiers manque). Le dos se
    /// relève sur leur gabarit, il ne se calcule pas.
    Mesure,
}

impl Dos {
    /// Épaisseur en mm, ou `None` quand le prestataire ne publie pas de formule.
    pub fn mm(&self, pages: u32) -> Option<f64> {
        let p = f64::from(pages);
        match *self {
            Dos::Divise { par, plus } => Some(p / par + plus),
            Dos::Multiplie { par, plus } => Some(p * par + plus),
            Dos::Mesure => None,
        }
    }
}

/// La seule géométrie de planche que l'application sache composer.
///
/// Une couverture rigide n'a ni le même gabarit, ni la même formule de dos : elle
/// déborde du livre, se replie à l'intérieur des plats et se monte sur des cartons.
/// Tant que `planche` ne sait pas la composer, aucune valeur ne la représente ici.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Geometrie {
    DosCarreColle,
}

/// La règle de parité que la composition sait appliquer.
///
/// Bookvault en impose une autre — multiple de douze moins un — que `interieur` ne sait
/// pas tenir. Elle n'a donc pas de valeur ici : son fichier écrit `paire`, qui est ce que
/// l'application fait, et la réserve est au COOKBOOK.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Parite {
    Paire,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Marges {
    pub haut: f64,
    pub bas: f64,
    /// Marge extérieure (sécurité), opposée à la gouttière.
    pub exterieur: f64,
}

/// Dimensions d'un format de rognage, en mm. Nommées et non positionnelles : ces fichiers
/// s'éditent à la main, et une largeur prise pour une hauteur donne un livre à l'italienne
/// que rien ne rattrape avant l'aperçu de la planche.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dimensions {
    pub largeur: f64,
    pub hauteur: f64,
}

/// Une tranche de pagination et la gouttière (marge intérieure) qu'elle impose.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tranche {
    pub de: u32,
    pub a: u32,
    pub mm: f64,
}

/// Pagination admise, bornes comprises.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pagination {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Format {
    pub cle: String,
    pub nom: String,
    /// **Transitoire.** La clé plate que portent encore le `.ozalid`, les répertoires de
    /// package et l'interface. Elle disparaît au lot 2, avec la migration des projets.
    pub cle_heritee: String,
    /// Format de rognage, en mm.
    pub mm: Dimensions,
    pub marges: Marges,
    /// Seules les tranches vérifiées dans le guide du prestataire figurent ici. Hors
    /// tranche, on refuse plutôt qu'inventer.
    pub gouttieres: Vec<Tranche>,
    /// Surcharge du fond perdu du POD, quand un format s'en écarte.
    #[serde(default)]
    pub fond_perdu: Option<f64>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reliure {
    pub cle: String,
    pub nom: String,
    /// Absente chez une reliure non outillée.
    #[serde(default)]
    pub geometrie: Option<Geometrie>,
    /// Pagination admise, bornes comprises. Elle vit sur la reliure et non sur le format :
    /// c'est elle qui la détermine — TheBookEdition accepte 40 à 750 pages en dos carré
    /// collé et 24 à 300 en rigide, au même format.
    #[serde(default)]
    pub pages: Option<Pagination>,
    #[serde(default)]
    pub parite: Option<Parite>,
    /// Pourquoi cette reliure n'est pas composable. Décrit **notre** état, jamais celui
    /// du POD : « géométrie non relevée » se vérifie, « le POD ne publie pas son rempli »
    /// serait une affirmation sur autrui qu'on n'a pas faite.
    #[serde(default)]
    pub non_outille: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finition {
    pub cle: String,
    pub nom: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Papier {
    pub cle: String,
    pub nom: String,
    /// La couleur du papier, en notation CSS, telle que le canevas la peint.
    ///
    /// **Convention d'Ozalid et non mesure** : aucun prestataire ne publie la teinte de
    /// son crème. Elle suit ce que le libellé annonce, et rien d'autre. Elle ne sert
    /// qu'à l'écran : le PDF n'a pas de fond, et lui en donner un ferait imprimer un
    /// aplat sur toutes les pages.
    pub teinte: String,
    pub dos: Dos,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pod {
    pub cle: String,
    pub nom: String,
    /// Fond perdu commun à ses formats, quand le POD le publie ainsi.
    #[serde(default)]
    pub fond_perdu: Option<f64>,
    #[serde(default, rename = "format")]
    pub formats: Vec<Format>,
    #[serde(default, rename = "reliure")]
    pub reliures: Vec<Reliure>,
    #[serde(default, rename = "finition")]
    pub finitions: Vec<Finition>,
    #[serde(default, rename = "papier")]
    pub papiers: Vec<Papier>,
}

/// Un nombre qu'on peut porter jusqu'au PDF, là où zéro n'a pas de sens : une dimension,
/// un facteur de dos. TOML 1.0 écrit `nan` et `inf` littéralement, un fichier peut donc
/// les contenir — et rien en aval ne rattrape un dos NaN, qui traverse la planche et
/// ressort dans la couverture remise à l'imprimeur.
fn fini_positif(v: f64) -> bool {
    v.is_finite() && v > 0.0
}

/// Le même contrôle pour ce qui a le droit d'être nul : une marge, un fond perdu, la
/// constante d'une formule de dos.
fn fini_non_negatif(v: f64) -> bool {
    v.is_finite() && v >= 0.0
}

/// Une clé du catalogue — POD, format, reliure, finition ou papier, la clé héritée
/// comprise — nomme un répertoire de package ou un identifiant : elle doit être un nom de
/// fichier et rien d'autre. Une clé vide ne se choisit pas dans l'interface et ne se
/// retrouve pas dans un `.ozalid` : elle ne désigne rien, et n'est donc pas davantage un
/// nom. `../../ailleurs` ou `C:nul*` s'écrivent sans peine dans un TOML, et `package` en
/// ferait un chemin.
fn est_un_nom(cle: &str) -> bool {
    !cle.is_empty()
        && cle
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Refuse deux entrées de même clé : le `find` des appelants en prendrait une des deux
/// sans que rien ne dise laquelle.
fn sans_doublon<'a>(
    pod: &str,
    quoi: &str,
    cles: impl Iterator<Item = &'a str>,
) -> Result<(), String> {
    let mut vues = BTreeSet::new();
    for cle in cles {
        if !vues.insert(cle) {
            return Err(format!("{pod} : deux {quoi} portent la clé « {cle} »."));
        }
    }
    Ok(())
}

impl Pod {
    /// Lit un POD depuis son TOML, et le refuse s'il promet ce que le code ne tient pas.
    pub fn depuis_toml(s: &str) -> Result<Self, String> {
        let pod: Pod = toml::from_str(s).map_err(|e| e.to_string())?;
        pod.verifie()?;
        Ok(pod)
    }

    fn verifie(&self) -> Result<(), String> {
        if !est_un_nom(&self.cle) {
            return Err(format!(
                "clé de POD « {} » : minuscules, chiffres et tirets, rien d'autre — elle \
                 nomme des répertoires et des identifiants.",
                self.cle
            ));
        }
        for (quoi, vide) in [
            ("format", self.formats.is_empty()),
            ("reliure", self.reliures.is_empty()),
            ("papier", self.papiers.is_empty()),
        ] {
            if vide {
                return Err(format!(
                    "{} : aucun bloc [[{quoi}]]. Choisir un livrable en suppose au moins \
                     un, et le premier papier fait le défaut.",
                    self.cle
                ));
            }
        }

        for (quoi, cle) in self
            .formats
            .iter()
            .map(|f| ("un format", f.cle.as_str()))
            .chain(
                self.reliures
                    .iter()
                    .map(|r| ("une reliure", r.cle.as_str())),
            )
            .chain(
                self.finitions
                    .iter()
                    .map(|f| ("une finition", f.cle.as_str())),
            )
            .chain(self.papiers.iter().map(|p| ("un papier", p.cle.as_str())))
        {
            if !est_un_nom(cle) {
                return Err(format!(
                    "{} : {quoi} à la clé « {cle} ». Minuscules, chiffres et tirets, rien \
                     d'autre — elle nomme des répertoires et des identifiants.",
                    self.cle
                ));
            }
        }
        for f in &self.formats {
            if !est_un_nom(&f.cle_heritee) {
                return Err(format!(
                    "{} / {} : clé héritée « {} ». Elle nomme un répertoire de package : \
                     minuscules, chiffres et tirets, rien d'autre.",
                    self.cle, f.cle, f.cle_heritee
                ));
            }
        }

        sans_doublon(
            &self.cle,
            "formats",
            self.formats.iter().map(|f| f.cle.as_str()),
        )?;
        // La clé héritée nomme un répertoire de package : deux formats qui la partagent
        // écriraient l'un sur l'autre.
        sans_doublon(
            &self.cle,
            "formats (clé héritée)",
            self.formats.iter().map(|f| f.cle_heritee.as_str()),
        )?;
        sans_doublon(
            &self.cle,
            "reliures",
            self.reliures.iter().map(|r| r.cle.as_str()),
        )?;
        sans_doublon(
            &self.cle,
            "finitions",
            self.finitions.iter().map(|f| f.cle.as_str()),
        )?;
        sans_doublon(
            &self.cle,
            "papiers",
            self.papiers.iter().map(|p| p.cle.as_str()),
        )?;

        if let Some(fp) = self.fond_perdu {
            if !fini_non_negatif(fp) {
                return Err(format!("{} : fond perdu impossible ({fp} mm).", self.cle));
            }
        }
        for f in &self.formats {
            self.verifie_format(f)?;
        }
        for r in &self.reliures {
            self.verifie_reliure(r)?;
        }
        for p in &self.papiers {
            self.verifie_papier(p)?;
        }

        // La table écrite en dur ne pouvait pas porter un tel POD ; un fichier déposé,
        // si. Sans ce refus il se lirait sans erreur, ne produirait aucune entrée plate,
        // et son imprimeur manquerait à la liste sans que rien ne le dise.
        if !self.reliures.iter().any(|r| r.geometrie.is_some()) {
            return Err(format!(
                "{} : aucune reliure composable. Un POD dont aucune reliure ne porte de \
                 géométrie ne produirait aucun format, et disparaîtrait sans un mot — \
                 donner geometrie = \"dos-carre-colle\" à sa reliure brochée.",
                self.cle
            ));
        }
        Ok(())
    }

    fn verifie_format(&self, f: &Format) -> Result<(), String> {
        let ou = format!("{} / {}", self.cle, f.cle);

        // Chaque valeur d'abord, les relations entre elles ensuite : une gouttière
        // infinie rend « marges plus larges que la page » vrai, et le message
        // désignerait alors le mauvais coupable.
        if !fini_positif(f.mm.largeur) || !fini_positif(f.mm.hauteur) {
            return Err(format!(
                "{ou} : format de rognage impossible ({} × {} mm).",
                f.mm.largeur, f.mm.hauteur
            ));
        }
        for (bord, v) in [
            ("haute", f.marges.haut),
            ("basse", f.marges.bas),
            ("extérieure", f.marges.exterieur),
        ] {
            if !fini_non_negatif(v) {
                return Err(format!("{ou} : marge {bord} impossible ({v} mm)."));
            }
        }
        if let Some(fp) = f.fond_perdu {
            if !fini_non_negatif(fp) {
                return Err(format!("{ou} : fond perdu impossible ({fp} mm)."));
            }
        }
        if f.gouttieres.is_empty() {
            return Err(format!(
                "{ou} : aucune tranche de gouttière. Un format qui n'en porte aucune ne \
                 compose aucune pagination."
            ));
        }
        for t in &f.gouttieres {
            if t.de > t.a {
                return Err(format!(
                    "{ou} : tranche de gouttière à l'envers ({}–{} pages).",
                    t.de, t.a
                ));
            }
            if t.de == 0 {
                return Err(format!(
                    "{ou} : tranche de gouttière à partir de zéro page. Un livre de zéro \
                     page n'existe pas plus qu'un format de zéro millimètre."
                ));
            }
            if !fini_non_negatif(t.mm) {
                return Err(format!("{ou} : gouttière impossible ({} mm).", t.mm));
            }
        }

        // Les relations. Des marges qui débordent la page laissent un bloc de texte nul
        // ou négatif, et l'intérieur composerait sans broncher une page fausse.
        if f.marges.haut + f.marges.bas >= f.mm.hauteur {
            return Err(format!(
                "{ou} : marges plus hautes que la page ({} + {} ≥ {} mm).",
                f.marges.haut, f.marges.bas, f.mm.hauteur
            ));
        }
        for (rang, t) in f.gouttieres.iter().enumerate() {
            // Deux tranches qui se recouvrent, et l'appelant prend la première qui
            // correspond : l'autre gouttière, relevée au guide, meurt sans un mot.
            for suivante in &f.gouttieres[rang + 1..] {
                if t.de <= suivante.a && suivante.de <= t.a {
                    return Err(format!(
                        "{ou} : deux tranches de gouttière se chevauchent ({}–{} et \
                         {}–{} pages).",
                        t.de, t.a, suivante.de, suivante.a
                    ));
                }
            }
            // La gouttière vit sur la tranche : la largeur utile s'y contrôle aussi.
            if f.marges.exterieur + t.mm >= f.mm.largeur {
                return Err(format!(
                    "{ou} : marges plus larges que la page ({} + {} ≥ {} mm) sur la \
                     tranche {}–{} pages.",
                    f.marges.exterieur, t.mm, f.mm.largeur, t.de, t.a
                ));
            }
        }
        Ok(())
    }

    fn verifie_reliure(&self, r: &Reliure) -> Result<(), String> {
        let ou = format!("{} / {}", self.cle, r.cle);
        // Piste notée, écartée ici : un `enum Outillage { Composable { … }, NonOutillee
        // { … } }` rendrait ces trois refus impossibles à commettre plutôt qu'à écrire.
        // Il change la forme des types que le lot 1 fixe et que ses appelants
        // consommeront ; à reprendre quand la vue plate aura disparu.
        match (&r.geometrie, &r.non_outille) {
            (None, None) => {
                return Err(format!(
                    "{ou} : ni géométrie ni raison de ne pas en avoir. Une reliure qu'on \
                     n'outille pas doit dire pourquoi."
                ))
            }
            (Some(_), Some(_)) => {
                return Err(format!(
                    "{ou} : une géométrie **et** une raison de ne pas en avoir. Elle sera \
                     composée par qui lit l'une, grisée par qui lit l'autre : au fichier \
                     de trancher."
                ))
            }
            (None, Some(_)) if r.pages.is_some() || r.parite.is_some() => {
                return Err(format!(
                    "{ou} : une reliure non outillée porte une pagination ou une parité \
                     qu'on n'appliquera pas — on contrôlerait une donnée dont on a décidé \
                     qu'elle ne sert pas. Ce qu'on garde pour mémoire s'écrit en \
                     commentaire TOML."
                ))
            }
            (Some(_), None) if r.pages.is_none() || r.parite.is_none() => {
                return Err(format!(
                    "{ou} : une reliure outillée doit porter sa pagination admise et sa \
                     parité."
                ))
            }
            _ => {}
        }
        if let Some(p) = r.pages {
            if p.min == 0 {
                return Err(format!(
                    "{ou} : pagination admise à partir de zéro page. Un livre de zéro \
                     page n'existe pas."
                ));
            }
            if p.min > p.max {
                return Err(format!(
                    "{ou} : pagination admise à l'envers ({}–{} pages).",
                    p.min, p.max
                ));
            }
        }
        Ok(())
    }

    fn verifie_papier(&self, p: &Papier) -> Result<(), String> {
        let ou = format!("{} / {}", self.cle, p.cle);
        // Non vide, et rien de plus : le doc dit « notation CSS », et restreindre à
        // `#rrggbb` rétrécirait un contrat écrit.
        if p.teinte.trim().is_empty() {
            return Err(format!(
                "{ou} : teinte vide. Le canevas la prendrait pour une couleur."
            ));
        }
        match p.dos {
            Dos::Divise { par, plus } | Dos::Multiplie { par, plus } => {
                if !fini_positif(par) {
                    return Err(format!("{ou} : facteur de dos impossible ({par})."));
                }
                if !fini_non_negatif(plus) {
                    return Err(format!("{ou} : constante de dos impossible ({plus} mm)."));
                }
            }
            Dos::Mesure => {}
        }
        Ok(())
    }
}

/// Les fichiers fournis, incorporés au binaire.
///
/// Par `include_str!` et non par lecture disque : l'immuabilité est un fait, pas une
/// règle applicative — il n'y a aucun fichier à protéger sur le poste, aucun chemin à
/// résoudre, aucun écart entre `cargo test` et l'application livrée. C'est le piège connu
/// de `fonts/`, où `target/debug` ne suit pas les sources.
const FOURNIS: &[&str] = &[
    include_str!("../pods/lulu.toml"),
    include_str!("../pods/bod.toml"),
    include_str!("../pods/kdp.toml"),
    include_str!("../pods/coollibri.toml"),
    include_str!("../pods/thebookedition.toml"),
    include_str!("../pods/bookvault.toml"),
];

/// Les POD fournis, dans l'ordre du tableau.
///
/// Une erreur ici n'est pas un cas d'usage mais un défaut de compilation logique : elle
/// remonte telle quelle, et le test `les_six_fichiers_fournis_se_lisent` est ce qui
/// l'attrape avant la livraison.
pub fn fournis() -> Result<Vec<Pod>, String> {
    FOURNIS.iter().map(|s| Pod::depuis_toml(s)).collect()
}

/// La vue **plate** du catalogue : une entrée par couple POD × format, telle que le reste
/// du code la consomme encore.
///
/// Transitoire dans son principe — le lot 2 lui substitue le livrable à cinq axes — mais
/// c'est elle qui rend ce lot invisible : rien d'autre ne change pendant qu'on déplace le
/// catalogue dans des fichiers.
#[derive(Debug, Clone, PartialEq)]
pub struct Provider {
    pub cle: String,
    pub libelle: String,
    pub format: (f64, f64),
    pub marge_haut: f64,
    pub marge_bas: f64,
    pub exterieur: f64,
    /// Triplets, comme la table historique les écrivait : ses appelants les lisent ainsi,
    /// et la vue plate n'est là que pour ne rien leur faire changer.
    pub gouttieres: Vec<(u32, u32, f64)>,
    pub fond_perdu: Option<f64>,
    pub pages_min: u32,
    pub pages_max: u32,
    pub papiers: Vec<Papier>,
}

impl Provider {
    /// Gouttière imposée par la tranche de pagination, en mm.
    pub fn gouttiere(&self, pages: u32) -> Result<f64, String> {
        self.gouttieres
            .iter()
            .find(|(lo, hi, _)| *lo <= pages && pages <= *hi)
            .map(|(_, _, g)| *g)
            .ok_or_else(|| {
                format!(
                    "{pages} pages : tranche de gouttière absente du gabarit {} — \
                     la compléter depuis le guide du prestataire.",
                    self.cle
                )
            })
    }

    /// Papier par défaut : le premier de la liste.
    pub fn papier_defaut(&self) -> &Papier {
        &self.papiers[0]
    }

    pub fn papier(&self, cle: &str) -> Option<&Papier> {
        self.papiers.iter().find(|p| p.cle == cle)
    }
}

/// Aplatit une liste de POD en une entrée par couple POD × format.
///
/// La reliure composable du POD donne la pagination admise ; un POD qui n'en aurait
/// aucune ne produit aucune entrée plate, faute de pouvoir dire ce qu'il accepte.
/// Qu'elle porte cette pagination est en revanche acquis : la lecture le vérifie.
pub fn aplatit(pods: &[Pod]) -> Vec<Provider> {
    let mut v = Vec::new();
    for pod in pods {
        let Some(r) = pod.reliures.iter().find(|r| r.geometrie.is_some()) else {
            continue;
        };
        let pagination = r
            .pages
            .expect("une reliure composable porte sa pagination : `verifie_reliure` la réclame");
        for f in &pod.formats {
            v.push(Provider {
                cle: f.cle_heritee.clone(),
                libelle: format!("{} — {}", pod.nom, f.nom),
                // La vue plate garde les tuples de la table historique : c'est ce qui
                // dispense `interieur` de traduire quoi que ce soit pour les lire.
                format: (f.mm.largeur, f.mm.hauteur),
                marge_haut: f.marges.haut,
                marge_bas: f.marges.bas,
                exterieur: f.marges.exterieur,
                gouttieres: f.gouttieres.iter().map(|t| (t.de, t.a, t.mm)).collect(),
                fond_perdu: f.fond_perdu.or(pod.fond_perdu),
                pages_min: pagination.min,
                pages_max: pagination.max,
                papiers: pod.papiers.clone(),
            });
        }
    }
    v
}

/// Le catalogue chargé, une fois pour la vie du processus.
///
/// `OnceLock` et non un état Tauri : deux signatures de `commands` rendent un
/// `&'static Provider`, et une table immuable chargée une fois les satisfait sans que
/// rien d'autre ne change. Hors application — les tests, le témoin —, il s'initialise
/// tout seul sur les seuls fournis.
///
/// Projection de `PODS` : `providers()` l'aplatit à la demande, il ne se pose plus lui-même.
/// Les deux tables viennent donc du même chargement **par construction**, et non plus
/// parce qu'`initialiser` les pose sur deux lignes voisines.
static PLATS: OnceLock<Vec<Provider>> = OnceLock::new();

/// Tous les couples POD × format connus.
pub fn providers() -> &'static [Provider] {
    PLATS.get_or_init(|| aplatit(pods()))
}

/// Le provider de cette clé, ou `None`.
pub fn provider(cle: &str) -> Option<&'static Provider> {
    providers().iter().find(|p| p.cle == cle)
}

/* ---------- l'identité d'un livrable ---------- */

/// L'identité de fabrication d'un livrable : les quatre axes qui changent le fichier
/// produit. La finition n'y est pas — mat ou brillant donnent le même PDF (spec § 4).
///
/// Sans `deny_unknown_fields`, **exprès** : la tâche 4 l'aplatit dans `Livrable` par
/// `#[serde(flatten)]`, que serde interdit de combiner avec `deny_unknown_fields` — et le
/// `flatten` rend de toute façon l'attribut inopérant sur les champs qu'il capture
/// (reconnaissance, verdict 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fabrication {
    pub pod: String,
    pub format: String,
    pub reliure: String,
    pub papier: String,
}

impl Fabrication {
    /// La clé du livrable : les quatre clés jointes par des tirets, telles quelles.
    /// Elle nomme le répertoire de package et l'identifiant de DOM. Elle se fabrique et
    /// se compare — jamais ne se découpe : le séparateur vit déjà dans les valeurs
    /// (`creme-90`).
    pub fn cle(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.pod, self.format, self.reliure, self.papier
        )
    }

    /// La clé du gabarit d'intérieur : ce qui détermine la pagination. Ni le papier ni
    /// la finition n'y sont — c'est elle qui range la mesure (spec § 5).
    pub fn cle_gabarit(&self) -> String {
        format!("{}-{}-{}", self.pod, self.format, self.reliure)
    }
}

/// La table des quatorze clés plates historiques et du triplet qui les remplace.
///
/// Deux maîtres : la migration v4→v5 des `.ozalid` (`projet::migre`), et le helper de
/// test `provider`. **Gelée** : un format neuf naît en triplet et n'y entre jamais, quand
/// bien même la vue plate grandirait au lot 4.
///
/// `expect(dead_code)` : cette tâche (1/lot 2) la pose sans encore la brancher hors tests —
/// `projet::migre` la consomme à la tâche 4, ce qui rendra l'attente non tenue et
/// l'attribut caduc de lui-même. Restreint à `not(test)` : `mod tests` l'utilise déjà, et
/// l'attente y serait déçue dans l'autre sens.
#[cfg_attr(not(test), expect(dead_code))]
pub(crate) const HERITEES: [(&str, &str, &str, &str); 14] = [
    ("lulu", "lulu", "108x175", "broche"),
    ("bod", "bod", "135x215", "broche"),
    ("kdp-5x8", "kdp", "5x8", "broche"),
    ("kdp-55x85", "kdp", "55x85", "broche"),
    ("kdp-6x9", "kdp", "6x9", "broche"),
    ("coollibri-110x170", "coollibri", "110x170", "broche"),
    ("coollibri-148x210", "coollibri", "148x210", "broche"),
    ("coollibri-160x240", "coollibri", "160x240", "broche"),
    ("tbe-110x170", "tbe", "110x170", "broche"),
    ("tbe-120x180", "tbe", "120x180", "broche"),
    ("tbe-1485x210", "tbe", "1485x210", "broche"),
    ("bookvault-127x203", "bookvault", "127x203", "broche"),
    ("bookvault-129x198", "bookvault", "129x198", "broche"),
    ("bookvault-148x210", "bookvault", "148x210", "broche"),
];

/// Les POD chargés, une fois pour la vie du processus — le pendant de `PLATS`, en
/// profondeur : c'est lui que `resout` interroge, et lui seul qu'`initialiser` pose ;
/// `PLATS` s'en déduit. Hors application, les seuls fournis.
static PODS: OnceLock<Vec<Pod>> = OnceLock::new();

/// Tous les POD connus.
pub fn pods() -> &'static [Pod] {
    PODS.get_or_init(|| fournis().expect("catalogue fourni illisible"))
}

/// Le POD de cette clé, ou `None`.
pub fn pod(cle: &str) -> Option<&'static Pod> {
    pods().iter().find(|p| p.cle == cle)
}

/// Un livrable résolu contre le catalogue : quatre références dans une table qui vit
/// aussi longtemps que le processus. `Copy` — rien ici n'est possédé.
#[derive(Debug, Clone, Copy)]
pub struct Resolu {
    pub pod: &'static Pod,
    pub format: &'static Format,
    pub reliure: &'static Reliure,
    pub papier: &'static Papier,
}

/// Résout une fabrication, ou la refuse en nommant l'axe fautif.
///
/// C'est ici que la reliure non outillée se refuse **par le Rust** (spec § 9), avec la
/// raison écrite dans le fichier : le refus tombe au moment du choix, jamais après une
/// couverture réglée.
pub fn resout(f: &Fabrication) -> Result<Resolu, String> {
    let pod = pod(&f.pod).ok_or_else(|| format!("POD inconnu : {}.", f.pod))?;
    let format = pod
        .formats
        .iter()
        .find(|x| x.cle == f.format)
        .ok_or_else(|| format!("{} ne fait pas le format {}.", pod.nom, f.format))?;
    let reliure = pod
        .reliures
        .iter()
        .find(|x| x.cle == f.reliure)
        .ok_or_else(|| format!("{} ne fait pas la reliure {}.", pod.nom, f.reliure))?;
    if reliure.geometrie.is_none() {
        return Err(match &reliure.non_outille {
            Some(raison) => format!("{} — {raison}", reliure.nom),
            None => format!("{} n'est pas composable.", reliure.nom),
        });
    }
    let papier = pod
        .papiers
        .iter()
        .find(|x| x.cle == f.papier)
        .ok_or_else(|| format!("papier inconnu chez {} : {}.", pod.nom, f.papier))?;
    Ok(Resolu {
        pod,
        format,
        reliure,
        papier,
    })
}

impl Resolu {
    /// Le fond perdu du format à défaut, celui du POD sinon — la règle d'`aplatit`.
    pub fn fond_perdu(&self) -> Option<f64> {
        self.format.fond_perdu.or(self.pod.fond_perdu)
    }

    /// La `Fabrication` d'où ce livrable vient : le chemin de retour de `Resolu` vers la
    /// clé à quatre segments, dont les tâches 3 à 6 auront besoin.
    pub fn fabrication(&self) -> Fabrication {
        Fabrication {
            pod: self.pod.cle.clone(),
            format: self.format.cle.clone(),
            reliure: self.reliure.cle.clone(),
            papier: self.papier.cle.clone(),
        }
    }

    /// La vue plate de ce livrable, telle que `interieur`, `planche` et `package` la
    /// consomment. Sa clé est celle du **gabarit** : c'est elle qui entre dans la source
    /// Typst et nomme le PDF de travail de `composer` — deux papiers du même gabarit
    /// composent le même intérieur.
    ///
    /// Le prix, nommable : quelques `String` et un `Vec<Papier>` clonés par commande,
    /// devant une composition Typst de plusieurs secondes (verdict 1c).
    pub fn provider(&self) -> Provider {
        let pagination = self
            .reliure
            .pages
            .expect("une reliure composable porte sa pagination : `verifie_reliure` la réclame");
        Provider {
            cle: self.fabrication().cle_gabarit(),
            libelle: format!("{} — {}", self.pod.nom, self.format.nom),
            format: (self.format.mm.largeur, self.format.mm.hauteur),
            marge_haut: self.format.marges.haut,
            marge_bas: self.format.marges.bas,
            exterieur: self.format.marges.exterieur,
            gouttieres: self
                .format
                .gouttieres
                .iter()
                .map(|t| (t.de, t.a, t.mm))
                .collect(),
            fond_perdu: self.fond_perdu(),
            pages_min: pagination.min,
            pages_max: pagination.max,
            papiers: self.pod.papiers.clone(),
        }
    }

    /// L'empreinte de ce qui pagine : format, marges, gouttières — rien d'autre.
    ///
    /// Retenue **avec la mesure** : un `<config>/pods/*.toml` réécrit avec d'autres
    /// marges ne périme la mesure qu'à travers elle (spec § 8). Le dos et le fond perdu
    /// n'y sont pas : ils ne paginent pas, et l'affichage les recalcule à chaque vue.
    pub fn empreinte(&self) -> String {
        let m = &self.format.marges;
        let g: Vec<String> = self
            .format
            .gouttieres
            .iter()
            .map(|t| format!("{}-{}-{}", t.de, t.a, t.mm))
            .collect();
        format!(
            "{}x{}|{}|{}|{}|{}",
            self.format.mm.largeur,
            self.format.mm.hauteur,
            m.haut,
            m.bas,
            m.exterieur,
            g.join(",")
        )
    }
}

/// Un fichier de catalogue que le poste porte et que l'application n'a pas pu lire.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Refus {
    pub fichier: String,
    pub raison: String,
}

/// Là où vivent les surcharges : à côté de `preferences.toml` et de `maquettes/`, parce
/// qu'elles appartiennent à la machine et non au livre.
fn repertoire(config: &Path) -> PathBuf {
    config.join("pods")
}

/// Les POD du binaire, puis ceux du poste. Même clé : le poste remplace, entièrement.
///
/// Une fusion champ par champ rendrait indéchiffrable ce que l'application lit vraiment :
/// devant une liste de formats, on ne saurait plus lesquels viennent du fichier déposé.
///
/// Deux fichiers du poste pour un même POD ne sont pas une faute — c'est le même
/// imprimeur — mais le dernier par nom de fichier l'emporte, et il vaut mieux le savoir
/// que le découvrir. Un fichier dont une clé héritée est déjà portée par un autre POD est
/// en revanche refusé : c'est elle qui nomme le prestataire dans le projet enregistré.
///
/// Rend aussi ce qui a été refusé, pour que l'interface puisse le dire. Un journal que
/// personne n'ouvre laisserait l'utilisateur devant un catalogue amputé.
pub fn charge(config: Option<&Path>) -> (Vec<Pod>, Vec<Refus>) {
    let mut pods = fournis().expect("catalogue fourni illisible");
    let mut refus = Vec::new();
    let Some(dir) = config.map(repertoire) else {
        return (pods, refus);
    };
    let entrees = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        // Un répertoire absent n'est pas une avarie : c'est l'état d'un poste où l'on
        // n'a rien déposé, c'est-à-dire presque tous.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (pods, refus),
        // Tout autre échec en est une, et le taire la ferait passer pour le cas
        // précédent : le poste croirait n'avoir rien déposé.
        Err(e) => {
            refus.push(Refus {
                fichier: dir.display().to_string(),
                raison: format!("répertoire de surcharges illisible : {e}"),
            });
            return (pods, refus);
        }
    };
    let mut chemins: Vec<_> = entrees
        .flatten()
        .map(|e| e.path())
        .filter(|c| c.extension().and_then(|x| x.to_str()) == Some("toml"))
        .collect();
    // Ordre du nom de fichier : deux postes identiques chargent identiquement, alors que
    // `read_dir` rend ses entrées dans un ordre que rien ne spécifie. Deux fichiers pour
    // un même POD sont donc départagés par leur nom, le dernier l'emportant.
    chemins.sort();
    for chemin in chemins {
        let nom = chemin.display().to_string();
        let lu = std::fs::read_to_string(&chemin)
            .map_err(|e| e.to_string())
            .and_then(|s| Pod::depuis_toml(&s));
        let pod = match lu {
            Ok(pod) => pod,
            Err(raison) => {
                refus.push(Refus {
                    fichier: nom,
                    raison,
                });
                continue;
            }
        };
        let remplace = pods.iter().position(|p| p.cle == pod.cle);
        // La clé héritée est celle que la vue plate porte, que le projet enregistre et
        // que `provider` cherche : deux POD qui la partagent, et le `find` en prend une
        // sans que rien ne dise laquelle — l'écran annoncerait un format pendant que le
        // dos et la pagination viendraient de l'autre. `sans_doublon` ne peut pas le
        // voir, c'est un fait entre POD. Le POD remplacé est écarté de la comparaison
        // avant elle, sans quoi un fichier reprenant les clés du fourni qu'il remplace se
        // refuserait lui-même.
        let collision = pods
            .iter()
            .enumerate()
            .filter(|(i, _)| Some(*i) != remplace)
            .flat_map(|(_, p)| p.formats.iter().map(move |f| (&f.cle_heritee, &p.cle)))
            .find(|(h, _)| pod.formats.iter().any(|f| &f.cle_heritee == *h))
            .map(|(h, c)| (h.clone(), c.clone()));
        if let Some((heritee, tenant)) = collision {
            refus.push(Refus {
                fichier: nom,
                raison: format!(
                    "{} : clé héritée « {heritee} » déjà portée par le POD « {tenant} ». \
                     Elle nomme le prestataire dans le projet enregistré : deux POD qui \
                     la partagent, et le livre composé n'est pas celui qu'annonce \
                     l'écran. En choisir une autre.",
                    pod.cle
                ),
            });
            continue;
        }
        match remplace {
            Some(i) => pods[i] = pod,
            None => pods.push(pod),
        }
    }
    (pods, refus)
}

/// Charge le catalogue une fois, au démarrage de l'application.
///
/// À appeler avant toute commande. Un second appel est un défaut d'ordonnancement et se
/// refuse : sans quoi les fichiers du poste seraient silencieusement ignorés, un
/// `providers()` antérieur ayant déjà figé les seuls fournis.
///
/// Ne pose que `PODS` : `PLATS` en est une projection, calculée à la demande par
/// `providers()`, jamais par lui.
pub fn initialiser(config: Option<&Path>) -> Result<Vec<Refus>, String> {
    let (pods, refus) = charge(config);
    PODS.set(pods)
        .map_err(|_| "le catalogue a déjà été chargé".to_string())?;
    Ok(refus)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    use tempfile::TempDir;

    // Le socle d'un POD valide. Les tests de refus n'écrivent que le bloc qu'ils mettent
    // en cause : sans ce socle ils échoueraient sur une liste vide, et leur assertion sur
    // le refus attendu ne dirait plus rien de ce qu'ils testent.
    const FORMAT: &str = r#"
[[format]]
cle = "135x215"
nom = "13,5 × 21,5 cm"
cle_heritee = "essai-135x215"
mm = { largeur = 135.0, hauteur = 215.0 }
marges = { haut = 18.8, bas = 28.0, exterieur = 15.0 }
gouttieres = [{ de = 24, a = 900, mm = 20.0 }]
"#;

    const RELIURE: &str = r#"
[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = { min = 24, max = 900 }
parite = "paire"
"#;

    const FINITION: &str = r#"
[[finition]]
cle = "mat"
nom = "Pelliculage mat"
"#;

    const PAPIER: &str = r##"
[[papier]]
cle = "creme-90"
nom = "Crème 90 g"
teinte = "#f7f0e0"
dos = { forme = "multiplie", par = 0.0675, plus = 0.6 }
"##;

    /// Assemble un POD à partir des blocs donnés. L'entête vient d'abord : en TOML, un
    /// scalaire écrit après une table lui appartiendrait.
    fn pod(blocs: &[&str]) -> String {
        format!(
            "cle = \"essai\"\nnom = \"Imprimeur d'essai\"\n{}",
            blocs.concat()
        )
    }

    /// Remplace un morceau d'un bloc du socle, en refusant de rendre le bloc intact : un
    /// gabarit qu'on retoucherait sans corriger le test le viderait de sa substance sans
    /// le faire échouer.
    ///
    /// Sa limite, pour qui s'en servira sur un cas plus retors : il garantit que l'aiguille
    /// a été trouvée, pas que le TOML obtenu dit ce qu'on croit. Un `avant` trop court
    /// frappe ailleurs qu'on ne pense, et un `apres` mal formé fait échouer le test sur une
    /// erreur de syntaxe qui ressemble à un refus. D'où l'assertion sur le message attendu,
    /// et non sur le seul fait qu'il y ait erreur.
    fn sauf(bloc: &str, avant: &str, apres: &str) -> String {
        let modifie = bloc.replace(avant, apres);
        assert_ne!(modifie, bloc, "« {avant} » ne figure plus dans le gabarit");
        modifie
    }

    /// Le TOML d'un POD se lit tel qu'il est écrit. Ce test tient la forme du format :
    /// s'il change, tous les fichiers fournis changent avec lui.
    #[test]
    fn un_pod_se_lit_depuis_son_toml() {
        let pod = Pod::depuis_toml(
            r##"
cle = "essai"
nom = "Imprimeur d'essai"
fond_perdu = 5.0

[[format]]
cle = "135x215"
nom = "13,5 × 21,5 cm"
cle_heritee = "essai-135x215"
mm = { largeur = 135.0, hauteur = 215.0 }
marges = { haut = 18.8, bas = 28.0, exterieur = 15.0 }
gouttieres = [{ de = 24, a = 900, mm = 20.0 }]

[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = { min = 24, max = 900 }
parite = "paire"

[[finition]]
cle = "mat"
nom = "Pelliculage mat"

[[papier]]
cle = "creme-90"
nom = "Crème 90 g"
teinte = "#f7f0e0"
dos = { forme = "multiplie", par = 0.0675, plus = 0.6 }
"##,
        )
        .unwrap();

        assert_eq!(pod.cle, "essai");
        assert_eq!(pod.fond_perdu, Some(5.0));
        assert_eq!(
            pod.formats[0].mm,
            Dimensions {
                largeur: 135.0,
                hauteur: 215.0
            }
        );
        assert_eq!(pod.formats[0].marges.bas, 28.0);
        assert_eq!(
            pod.formats[0].gouttieres,
            vec![Tranche {
                de: 24,
                a: 900,
                mm: 20.0
            }]
        );
        assert_eq!(pod.reliures[0].geometrie, Some(Geometrie::DosCarreColle));
        assert_eq!(
            pod.reliures[0].pages,
            Some(Pagination { min: 24, max: 900 })
        );
        // La finition ne porte rien d'autre que son nom, et c'est ce nom que l'interface
        // affiche : rien d'autre ne le lirait si ce test ne le lisait pas.
        assert_eq!(pod.finitions[0].cle, "mat");
        assert_eq!(pod.finitions[0].nom, "Pelliculage mat");
        assert_eq!(pod.papiers[0].teinte, "#f7f0e0");
        // Comparaison à la tolérance : 0,0675 n'a pas de représentation binaire
        // exacte, et `280 × 0,0675 + 0,6` ne vaut pas `19.5` au bit près.
        let dos = pod.papiers[0].dos.mm(280).unwrap();
        assert!((dos - 19.5).abs() < 1e-9, "dos {dos}");
    }

    /// Une géométrie que le code ne sait pas appliquer est **refusée**, jamais ignorée.
    /// C'est l'énumération fermée qui la refuse, avant même `verifie` : lui ajouter un
    /// `#[serde(other)]` ferait tomber ce test, et c'est exactement ce qu'il protège —
    /// le fichier de données ne doit pas pouvoir promettre une reliure que la planche ne
    /// compose pas.
    #[test]
    fn une_geometrie_inconnue_est_refusee_en_la_nommant() {
        let r = sauf(RELIURE, "dos-carre-colle", "cousue");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("cousue"), "{e}");
    }

    /// Le pendant, pour l'autre énumération fermée. Bookvault impose un multiple de douze
    /// moins un, que la composition ne sait pas tenir : le jour où un fichier l'écrirait,
    /// il doit être refusé et non lu comme « paire ».
    #[test]
    fn une_parite_inconnue_est_refusee_en_la_nommant() {
        let r = sauf(RELIURE, "\"paire\"", "\"multiple-12-moins-1\"");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("multiple-12-moins-1"), "{e}");
    }

    /// Une reliure qui n'annonce ni géométrie ni raison de ne pas en avoir est un oubli,
    /// pas un choix : on ne peut pas deviner si elle est composable.
    #[test]
    fn une_reliure_sans_geometrie_ni_raison_est_refusee() {
        let rigide = r#"
[[reliure]]
cle = "rigide"
nom = "Couverture rigide"
"#;
        let e = Pod::depuis_toml(&pod(&[FORMAT, rigide, PAPIER])).unwrap_err();
        assert!(e.contains("rigide"), "{e}");
        assert!(e.contains("ni géométrie ni raison"), "{e}");
    }

    /// Une reliure qui porte à la fois sa géométrie et sa raison de ne pas en avoir dit
    /// deux choses contraires : l'appelant qui interroge `geometrie` la compose, celui qui
    /// interroge `non_outille` la grise. C'est au fichier de trancher, pas à eux.
    #[test]
    fn une_reliure_a_la_fois_outillee_et_non_outillee_est_refusee() {
        let r = format!("{RELIURE}non_outille = \"géométrie non relevée\"\n");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("broche"), "{e}");
        assert!(e.contains("une raison de ne pas en avoir"), "{e}");
    }

    /// Le chemin qu'aucun autre test ne parcourt : une reliure non outillée est **lue**,
    /// pas refusée. C'est elle qui doit paraître grisée avec sa raison en clair ; un
    /// contrôle trop zélé la ferait disparaître du catalogue entier.
    #[test]
    fn une_reliure_non_outillee_est_lue_avec_sa_raison() {
        let rigide = r#"
[[reliure]]
cle = "rigide"
nom = "Couverture rigide"
non_outille = "géométrie du casewrap non relevée : rempli, mors, épaisseur des cartons"
"#;
        let p = Pod::depuis_toml(&pod(&[FORMAT, RELIURE, rigide, PAPIER])).unwrap();
        assert_eq!(p.reliures[1].geometrie, None);
        let raison = p.reliures[1].non_outille.as_deref().unwrap();
        assert!(raison.contains("casewrap"), "{raison}");
    }

    /// Mais elle ne porte ni pagination ni parité : elles seraient contrôlées puis
    /// ignorées, ce qui est le comble — on vérifierait une donnée dont on a décidé
    /// qu'elle ne sert pas. Ce qu'on garde pour mémoire s'écrit en commentaire TOML.
    #[test]
    fn une_reliure_non_outillee_ne_porte_ni_pagination_ni_parite() {
        for ligne in ["pages = { min = 24, max = 300 }", "parite = \"paire\""] {
            let rigide = format!(
                r#"
[[reliure]]
cle = "rigide"
nom = "Couverture rigide"
non_outille = "géométrie du casewrap non relevée"
{ligne}
"#
            );
            let e = Pod::depuis_toml(&pod(&[FORMAT, &rigide, PAPIER])).unwrap_err();
            assert!(e.contains("rigide"), "{ligne} : {e}");
            assert!(e.contains("qu'on n'appliquera pas"), "{ligne} : {e}");
        }
    }

    /// Une reliure outillée sans pagination admise laisserait `package` accepter
    /// n'importe quel compte de pages : le refus de pagination est un contrôle, pas une
    /// décoration.
    #[test]
    fn une_reliure_outillee_sans_pagination_est_refusee() {
        let r = sauf(RELIURE, "pages = { min = 24, max = 900 }\n", "");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("broche"), "{e}");
        assert!(e.contains("pagination admise et sa parité"), "{e}");
    }

    /// Même chose pour la parité : sans elle, `interieur` ne sait pas s'il doit ajouter
    /// une blanche de fin, et la composition perd la seule règle qu'elle sache tenir.
    #[test]
    fn une_reliure_outillee_sans_parite_est_refusee() {
        let r = sauf(RELIURE, "parite = \"paire\"\n", "");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("broche"), "{e}");
        assert!(e.contains("pagination admise et sa parité"), "{e}");
    }

    /// Un facteur de dos non fini, nul ou négatif produit un dos NaN, infini ou rentrant,
    /// et rien plus loin ne le rattrape : la planche le porte tel quel jusqu'au PDF de
    /// couverture. TOML 1.0 écrit `nan` et `inf` littéralement — ce n'est pas une valeur
    /// qu'un fichier serait incapable de contenir.
    #[test]
    fn une_formule_de_dos_impossible_est_refusee() {
        for par in ["nan", "inf", "0.0", "-0.07"] {
            let p = sauf(PAPIER, "par = 0.0675", &format!("par = {par}"));
            let e = Pod::depuis_toml(&pod(&[FORMAT, RELIURE, &p])).unwrap_err();
            assert!(e.contains("creme-90"), "par = {par} : {e}");
            assert!(e.contains("facteur de dos impossible"), "par = {par} : {e}");
        }
        for plus in ["nan", "-1.0"] {
            let p = sauf(PAPIER, "plus = 0.6", &format!("plus = {plus}"));
            let e = Pod::depuis_toml(&pod(&[FORMAT, RELIURE, &p])).unwrap_err();
            assert!(e.contains("creme-90"), "plus = {plus} : {e}");
            assert!(
                e.contains("constante de dos impossible"),
                "plus = {plus} : {e}"
            );
        }
    }

    /// Une teinte vide ne peint rien, et le canevas la prendrait pour une couleur. On
    /// n'exige pas davantage : le contrat écrit dit « notation CSS », et le restreindre à
    /// `#rrggbb` rétrécirait ce qu'on a promis.
    #[test]
    fn une_teinte_vide_est_refusee() {
        let p = sauf(PAPIER, "teinte = \"#f7f0e0\"", "teinte = \"\"");
        let e = Pod::depuis_toml(&pod(&[FORMAT, RELIURE, &p])).unwrap_err();
        assert!(e.contains("creme-90"), "{e}");
        assert!(e.contains("teinte vide"), "{e}");
    }

    /// Même refus pour la géométrie de la page : un format de rognage nul ou non fini, une
    /// marge négative, un fond perdu impossible traversent tout aussi loin — jusqu'au
    /// gabarit de l'intérieur, où ils ne provoquent pas une erreur mais une page fausse.
    /// Chaque bord est éprouvé : une boucle qui n'en contrôlerait qu'un passerait.
    #[test]
    fn une_dimension_ou_une_marge_impossible_est_refusee() {
        for (avant, apres, attendu) in [
            (
                "largeur = 135.0",
                "largeur = 0.0",
                "format de rognage impossible",
            ),
            (
                "hauteur = 215.0",
                "hauteur = nan",
                "format de rognage impossible",
            ),
            ("haut = 18.8", "haut = nan", "marge haute impossible"),
            ("bas = 28.0", "bas = -28.0", "marge basse impossible"),
            (
                "exterieur = 15.0",
                "exterieur = -1.0",
                "marge extérieure impossible",
            ),
            ("mm = 20.0", "mm = inf", "gouttière impossible"),
        ] {
            let f = sauf(FORMAT, avant, apres);
            let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
            assert!(e.contains(attendu), "{apres} : {e}");
        }

        // Le fond perdu s'écrit au POD et se surcharge au format : les deux se contrôlent.
        let e =
            Pod::depuis_toml(&pod(&["fond_perdu = -5.0\n", FORMAT, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("fond perdu impossible"), "{e}");
        let f = format!("{FORMAT}fond_perdu = nan\n");
        let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
        assert!(
            e.contains("135x215") && e.contains("fond perdu impossible"),
            "{e}"
        );
    }

    /// Des marges qui débordent la page laissent un bloc de texte nul ou négatif.
    /// `interieur` ne lèvera pas d'erreur pour autant : il composera une page fausse. La
    /// gouttière vit sur la tranche, le contrôle de largeur s'y fait donc aussi.
    #[test]
    fn une_marge_plus_grande_que_la_page_est_refusee() {
        let f = sauf(FORMAT, "exterieur = 15.0", "exterieur = 500.0");
        let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("plus larges que la page"), "{e}");

        let f = sauf(
            FORMAT,
            "haut = 18.8, bas = 28.0",
            "haut = 200.0, bas = 200.0",
        );
        let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("plus hautes que la page"), "{e}");
    }

    /// Une tranche dont la borne basse dépasse la haute n'admet aucune valeur : elle
    /// refuserait toute pagination, ou n'appliquerait jamais sa gouttière. C'est une
    /// coquille de saisie, et elle doit se voir au chargement plutôt qu'à la composition.
    #[test]
    fn une_tranche_a_l_envers_est_refusee() {
        let r = sauf(RELIURE, "min = 24, max = 900", "min = 900, max = 24");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("broche") && e.contains("à l'envers"), "{e}");

        let f = sauf(FORMAT, "de = 24, a = 900", "de = 900, a = 24");
        let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("135x215") && e.contains("à l'envers"), "{e}");
    }

    /// Un livre de zéro page n'existe pas plus qu'un format de zéro millimètre : les
    /// entiers ont le même plancher que les flottants, sans quoi une tranche partant de
    /// zéro promettrait une pagination que rien ne compose.
    #[test]
    fn une_pagination_qui_part_de_zero_est_refusee() {
        let r = sauf(RELIURE, "min = 24", "min = 0");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("broche") && e.contains("zéro page"), "{e}");

        let f = sauf(FORMAT, "de = 24", "de = 0");
        let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("135x215") && e.contains("zéro page"), "{e}");
    }

    /// Deux tranches qui se chevauchent, et l'appelant prend la première qui correspond :
    /// la seconde gouttière, relevée au guide, meurt sans un mot. C'est l'argument de
    /// `sans_doublon`, appliqué aux tranches.
    ///
    /// Le second cas tient le **côté accepté**, qui compte autant : deux tranches qui
    /// s'enchaînent sans se recouvrir sont la forme réelle de KDP. Un glissement de la
    /// borne refuserait un catalogue correct, et rien ne le dirait.
    #[test]
    fn deux_tranches_de_gouttiere_qui_se_chevauchent_sont_refusees() {
        let f = sauf(
            FORMAT,
            "gouttieres = [{ de = 24, a = 900, mm = 20.0 }]",
            "gouttieres = [{ de = 24, a = 900, mm = 20.0 }, { de = 100, a = 200, mm = 15.0 }]",
        );
        let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("135x215") && e.contains("se chevauchent"), "{e}");

        // La paire réelle de KDP : 24–700 puis 701–828, bornes jointives.
        let f = sauf(
            FORMAT,
            "gouttieres = [{ de = 24, a = 900, mm = 20.0 }]",
            "gouttieres = [{ de = 24, a = 700, mm = 20.0 }, { de = 701, a = 828, mm = 22.0 }]",
        );
        let lu = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap();
        assert_eq!(lu.formats[0].gouttieres.len(), 2);
    }

    /// Un format sans aucune tranche ne compose aucune pagination : c'est la même
    /// amputation qu'un POD sans format, et elle se refuse pareillement.
    #[test]
    fn un_format_sans_tranche_de_gouttiere_est_refuse() {
        let f = sauf(
            FORMAT,
            "gouttieres = [{ de = 24, a = 900, mm = 20.0 }]",
            "gouttieres = []",
        );
        let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
        assert!(
            e.contains("135x215") && e.contains("aucune tranche de gouttière"),
            "{e}"
        );
    }

    /// Une clé vide ne se choisit pas et ne se retrouve pas dans un `.ozalid`. La clé
    /// héritée, elle, nomme un répertoire de package : `../../ailleurs` s'écrit sans peine
    /// dans un TOML, et `package` en ferait un chemin.
    #[test]
    fn une_cle_vide_ou_qui_n_est_pas_un_nom_est_refusee() {
        for (bloc, avant, quoi) in [
            (FORMAT, "cle = \"135x215\"", "un format"),
            (RELIURE, "cle = \"broche\"", "une reliure"),
            (FINITION, "cle = \"mat\"", "une finition"),
            (PAPIER, "cle = \"creme-90\"", "un papier"),
        ] {
            let ampute = sauf(bloc, avant, "cle = \"\"");
            let blocs: Vec<&str> = [FORMAT, RELIURE, FINITION, PAPIER]
                .iter()
                .map(|b| if *b == bloc { ampute.as_str() } else { *b })
                .collect();
            let e = Pod::depuis_toml(&pod(&blocs)).unwrap_err();
            let attendu = format!("{quoi} à la clé");
            assert!(e.contains(&attendu), "attendu « {attendu} », reçu : {e}");
        }

        for cle_heritee in ["", "../../ailleurs", "ailleurs/essai", "Essai 135x215"] {
            let f = sauf(
                FORMAT,
                "cle_heritee = \"essai-135x215\"",
                &format!("cle_heritee = \"{cle_heritee}\""),
            );
            let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
            assert!(e.contains("clé héritée"), "{cle_heritee} : {e}");
        }
    }

    /// Le trou du verdict 4 : une clé de POD qui n'est pas un nom se lisait sans erreur,
    /// alors qu'elle nomme un répertoire de package à partir du lot 2.
    #[test]
    fn une_cle_de_pod_qui_n_est_pas_un_nom_est_refusee() {
        let e = Pod::depuis_toml(&format!(
            r##"cle = "../evade"
nom = "Essai"
{FORMAT}{RELIURE}{PAPIER}"##
        ))
        .unwrap_err();
        assert!(e.contains("../evade"), "{e}");
    }

    /// Même trou côté papier : `../../ailleurs` s'écrit sans peine dans un TOML.
    #[test]
    fn une_cle_de_papier_qui_n_est_pas_un_nom_est_refusee() {
        let e = Pod::depuis_toml(&format!(
            r##"cle = "essai"
nom = "Essai"
{FORMAT}{RELIURE}
[[papier]]
cle = "../../ailleurs"
nom = "Papier"
teinte = "#ffffff"
dos = {{ forme = "multiplie", par = 0.06, plus = 0.0 }}
"##
        ))
        .unwrap_err();
        assert!(e.contains("../../ailleurs"), "{e}");
    }

    /// Et côté format : `C:nul*` n'est ni un nom de répertoire, ni un nom de fichier.
    #[test]
    fn une_cle_de_format_qui_n_est_pas_un_nom_est_refusee() {
        let e = Pod::depuis_toml(&format!(
            r##"cle = "essai"
nom = "Essai"
[[format]]
cle = "C:nul*"
nom = "Format"
cle_heritee = "essai"
mm = {{ largeur = 100.0, hauteur = 100.0 }}
marges = {{ haut = 10.0, bas = 10.0, exterieur = 10.0 }}
gouttieres = [{{ de = 1, a = 900, mm = 10.0 }}]
{RELIURE}{PAPIER}"##
        ))
        .unwrap_err();
        assert!(e.contains("C:nul*"), "{e}");
    }

    /// Un champ que le catalogue ne connaît pas est refusé, jamais ignoré. `fond-perdu`
    /// avec un tiret, ou `fond_perdue`, sont des valeurs que quelqu'un a relevées et
    /// écrites : les passer sous silence ferait dire au catalogue ce qu'il n'a pas lu.
    #[test]
    fn un_champ_inconnu_est_refuse_en_le_nommant() {
        let e =
            Pod::depuis_toml(&pod(&["fond-perdu = 5.0\n", FORMAT, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("fond-perdu"), "{e}");

        // Y compris dans une table en ligne, où le champ de trop se remarque moins.
        let f = sauf(
            FORMAT,
            "largeur = 135.0",
            "largeur = 135.0, profondeur = 1.0",
        );
        let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("profondeur"), "{e}");

        // Y compris dans la formule de dos, qui est celle qui porte les chiffres.
        let p = sauf(
            PAPIER,
            "plus = 0.6 }",
            "plus = 0.6, soucre = \"le 20/08\" }",
        );
        let e = Pod::depuis_toml(&pod(&[FORMAT, RELIURE, &p])).unwrap_err();
        assert!(e.contains("soucre"), "{e}");
    }

    /// Deux entrées de même clé, et le `find` des appelants en prend une sans que rien ne
    /// dise laquelle — deux papiers homonymes aux dos différents donneraient deux
    /// épaisseurs selon l'appelant. Les cinq listes se contrôlent, la clé héritée comprise :
    /// elle nomme un répertoire de package.
    #[test]
    fn deux_cles_identiques_sont_refusees() {
        let autre = sauf(FORMAT, "cle = \"135x215\"", "cle = \"148x210\"");
        for (blocs, attendu) in [
            (
                vec![FORMAT, FORMAT, RELIURE, PAPIER],
                "deux formats portent la clé « 135x215 »",
            ),
            (
                vec![FORMAT, autre.as_str(), RELIURE, PAPIER],
                "deux formats (clé héritée) portent la clé « essai-135x215 »",
            ),
            (
                vec![FORMAT, RELIURE, RELIURE, PAPIER],
                "deux reliures portent la clé « broche »",
            ),
            (
                vec![FORMAT, RELIURE, FINITION, FINITION, PAPIER],
                "deux finitions portent la clé « mat »",
            ),
            (
                vec![FORMAT, RELIURE, PAPIER, PAPIER],
                "deux papiers portent la clé « creme-90 »",
            ),
        ] {
            let e = Pod::depuis_toml(&pod(&blocs)).unwrap_err();
            assert!(e.contains(attendu), "attendu « {attendu} », reçu : {e}");
        }
    }

    /// `papier_defaut()` indexera `papiers[0]`, et le choix d'un format comme d'une
    /// reliure suppose qu'il en existe au moins un. L'invariant que `&'static [Papier]`
    /// tenait par construction n'a plus que le chargement où vivre : un POD amputé doit
    /// être refusé, pas faire paniquer l'application au premier clic. Le message doit
    /// nommer **ce qui manque** : c'est lui qui dit quoi faire.
    #[test]
    fn un_pod_sans_format_reliure_ou_papier_est_refuse() {
        for (manquant, blocs) in [
            ("format", [RELIURE, PAPIER]),
            ("reliure", [FORMAT, PAPIER]),
            ("papier", [FORMAT, RELIURE]),
        ] {
            let e = Pod::depuis_toml(&pod(&blocs)).unwrap_err();
            let attendu = format!("aucun bloc [[{manquant}]]");
            assert!(e.contains(&attendu), "attendu « {attendu} », reçu : {e}");
        }
    }

    /// Les deux formes que le premier test ne calcule pas. `Divise` est la formule de
    /// Lulu, en production : intervertir la division et la multiplication ne casserait
    /// rien sans ce test. `Mesure` doit rendre `None` et non zéro — un dos qu'on ne sait
    /// pas calculer n'est pas un dos plat.
    #[test]
    fn le_dos_se_calcule_ou_s_abstient_selon_sa_forme() {
        let p = sauf(
            PAPIER,
            "forme = \"multiplie\", par = 0.0675, plus = 0.6",
            "forme = \"divise\", par = 17.48, plus = 1.524",
        );
        let lu = Pod::depuis_toml(&pod(&[FORMAT, RELIURE, &p])).unwrap();
        // Formule Lulu, vérifiée sur un livre réel de 244 pages → 15,48 mm.
        let dos = lu.papiers[0].dos.mm(244).unwrap();
        assert!((dos - 15.48).abs() < 0.01, "dos {dos}");

        let p = sauf(
            PAPIER,
            "{ forme = \"multiplie\", par = 0.0675, plus = 0.6 }",
            "{ forme = \"mesure\" }",
        );
        let lu = Pod::depuis_toml(&pod(&[FORMAT, RELIURE, &p])).unwrap();
        assert_eq!(lu.papiers[0].dos, Dos::Mesure);
        assert_eq!(lu.papiers[0].dos.mm(244), None);
    }

    /// Les six fournis se lisent tous. Un TOML mal formé ne casse plus la compilation mais
    /// le démarrage : ce test est ce qui le rattrape avant la livraison.
    #[test]
    fn les_six_fichiers_fournis_se_lisent() {
        let pods = fournis().expect("un fichier fourni est illisible");
        assert_eq!(pods.len(), 6, "six POD attendus");
        // Les quatorze formats de la table historique, tous présents.
        let formats: usize = pods.iter().map(|p| p.formats.len()).sum();
        assert_eq!(formats, 14, "quatorze formats attendus");
    }

    /// Chaque POD outillé porte au moins un papier et une reliure composable : sans quoi il
    /// serait en table sans que rien ne puisse en sortir.
    #[test]
    fn chaque_pod_fourni_porte_un_papier_et_une_reliure_composable() {
        for p in fournis().unwrap() {
            assert!(!p.papiers.is_empty(), "{} sans papier", p.cle);
            assert!(
                p.reliures.iter().any(|r| r.geometrie.is_some()),
                "{} sans reliure composable",
                p.cle
            );
        }
    }

    // Les six tests qui suivent portent sur le chargement depuis le poste. Tous passent
    // par `charge`, jamais par `initialiser` : `PLATS` est un `OnceLock` de processus
    // que des dizaines d'autres tests ont déjà rempli en appelant `provider(…)`, et un
    // test d'`initialiser` réussirait ou échouerait selon l'ordre d'exécution.

    /// Écrit un fichier de POD dans le répertoire de surcharges d'un poste d'essai.
    fn pose(dir: &TempDir, nom: &str, contenu: &str) {
        let d = dir.path().join("pods");
        std::fs::create_dir_all(&d).unwrap();
        let mut f = std::fs::File::create(d.join(nom)).unwrap();
        f.write_all(contenu.as_bytes()).unwrap();
    }

    const IMPRIMEUR_ESSAI: &str = r##"
cle = "essai"
nom = "Imprimeur d'essai"
fond_perdu = 4.0

[[format]]
cle = "100x150"
nom = "10 × 15 cm"
cle_heritee = "essai"
mm = { largeur = 100.0, hauteur = 150.0 }
marges = { haut = 10.0, bas = 10.0, exterieur = 10.0 }
gouttieres = [ { de = 24, a = 400, mm = 15.0 } ]

[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = { min = 24, max = 400 }
parite = "paire"

[[papier]]
cle = "standard"
nom = "Papier standard"
teinte = "#ffffff"
dos = { forme = "multiplie", par = 0.06, plus = 0.0 }
"##;

    /// Un POD que le binaire ne connaît pas s'ajoute par un fichier déposé. C'est tout
    /// l'objet du chantier : un imprimeur de plus ne demande pas de relivrer l'application.
    #[test]
    fn un_fichier_du_poste_ajoute_un_pod() {
        let d = TempDir::new().unwrap();
        pose(&d, "essai.toml", IMPRIMEUR_ESSAI);
        let (pods, refus) = charge(Some(d.path()));
        assert!(refus.is_empty(), "{refus:?}");
        assert_eq!(pods.len(), 7);
        assert!(pods.iter().any(|p| p.cle == "essai"));
    }

    /// Même clé : le fichier du poste remplace le fourni **entièrement**. Une fusion champ
    /// par champ rendrait indéchiffrable ce que l'application lit vraiment.
    #[test]
    fn un_fichier_du_poste_remplace_le_fourni_de_meme_cle() {
        let d = TempDir::new().unwrap();
        pose(
            &d,
            "bod.toml",
            &IMPRIMEUR_ESSAI.replace(r#"cle = "essai""#, r#"cle = "bod""#),
        );
        let (pods, refus) = charge(Some(d.path()));
        assert!(refus.is_empty(), "{refus:?}");
        assert_eq!(pods.len(), 6, "un remplacement, pas un ajout");
        let bod = pods.iter().find(|p| p.cle == "bod").unwrap();
        assert_eq!(bod.fond_perdu, Some(4.0), "le fourni tient encore");
        assert_eq!(bod.formats.len(), 1);
        assert_eq!(
            (bod.formats[0].mm.largeur, bod.formats[0].mm.hauteur),
            (100.0, 150.0)
        );
    }

    /// Un fichier fautif est refusé **en le nommant**, et les autres se chargent quand même.
    /// L'application démarre toujours : un catalogue amputé sans explication laisserait
    /// l'utilisateur devant une liste incomplète sans savoir pourquoi.
    #[test]
    fn un_fichier_fautif_est_refuse_en_le_nommant_et_les_autres_tiennent() {
        let d = TempDir::new().unwrap();
        pose(&d, "casse.toml", "cle = \"casse\"\nnom =");
        pose(&d, "essai.toml", IMPRIMEUR_ESSAI);
        let (pods, refus) = charge(Some(d.path()));
        assert_eq!(refus.len(), 1);
        assert!(refus[0].fichier.contains("casse.toml"), "{:?}", refus[0]);
        assert!(!refus[0].raison.is_empty());
        assert!(
            pods.iter().any(|p| p.cle == "essai"),
            "le fichier sain n'a pas été chargé"
        );
        assert!(
            pods.iter().any(|p| p.cle == "bod"),
            "les fournis n'ont pas tenu"
        );
    }

    /// Un POD sans reliure composable ne produit aucune entrée : il doit le dire, pas
    /// s'évanouir.
    ///
    /// `aplatit` ne retient qu'un POD portant une reliure de géométrie connue — c'est
    /// délibéré, on ne peut pas annoncer un format qu'on ne sait pas composer. Mais tant que
    /// le catalogue était écrit en dur, le cas n'existait pas ; un fichier déposé, si. Sans
    /// ce refus, l'utilisateur dépose un fichier valide, relance, et son imprimeur n'est
    /// nulle part — sans un mot.
    #[test]
    fn un_pod_sans_reliure_composable_est_refuse() {
        let d = TempDir::new().unwrap();
        pose(
            &d,
            "rigide.toml",
            &IMPRIMEUR_ESSAI.replace(
                "geometrie = \"dos-carre-colle\"\npages = { min = 24, max = 400 }\nparite = \"paire\"",
                "non_outille = \"géométrie du casewrap non relevée\"",
            ),
        );
        let (pods, refus) = charge(Some(d.path()));
        assert_eq!(refus.len(), 1, "{pods:?}");
        assert!(refus[0].raison.contains("reliure"), "{:?}", refus[0]);
        // Un message qui dit ce qui se passerait sans dire quoi faire laisse
        // l'utilisateur devant son fichier, sans savoir quelle ligne écrire.
        assert!(refus[0].raison.contains("geometrie"), "{:?}", refus[0]);
    }

    /// Un répertoire de surcharges absent n'est pas une avarie : c'est l'état d'un poste où
    /// l'on n'a rien déposé.
    #[test]
    fn un_repertoire_absent_charge_les_seuls_fournis() {
        let d = TempDir::new().unwrap();
        let (pods, refus) = charge(Some(d.path()));
        assert!(refus.is_empty());
        assert_eq!(pods.len(), 6);
    }

    /// Deux POD qui se disputent une clé héritée : le pire cas que ce chantier rend
    /// atteignable. La clé héritée est celle que la vue plate porte, que le `.ozalid`
    /// enregistre et que `provider` cherche — deux entrées de même clé, et le `find` en
    /// prend une sans que rien ne dise laquelle. L'écran annoncerait 10 × 15 pendant que
    /// le dos, la pagination et les papiers seraient ceux de BoD. Découvert au tirage.
    #[test]
    fn une_cle_heritee_deja_prise_par_un_autre_pod_est_refusee() {
        let d = TempDir::new().unwrap();
        pose(
            &d,
            "essai.toml",
            &IMPRIMEUR_ESSAI.replace(r#"cle_heritee = "essai""#, r#"cle_heritee = "bod""#),
        );
        let (pods, refus) = charge(Some(d.path()));
        assert_eq!(refus.len(), 1, "{pods:?}");
        assert!(refus[0].fichier.contains("essai.toml"), "{:?}", refus[0]);
        assert!(refus[0].raison.contains("bod"), "{:?}", refus[0]);
        assert!(
            !pods.iter().any(|p| p.cle == "essai"),
            "le fichier fautif a été retenu quand même"
        );
        // Deux entrées de même clé dans la vue plate, c'est exactement ce qu'on empêche.
        let plats = aplatit(&pods);
        assert_eq!(plats.iter().filter(|p| p.cle == "bod").count(), 1);
    }

    /// Un fichier qui remplace un fourni reprend ses clés héritées : il ne doit pas se
    /// refuser sur les siennes. C'est l'ordre du contrôle qui le décide — ôter d'abord le
    /// remplacé, comparer ensuite.
    #[test]
    fn un_remplacement_ne_se_refuse_pas_sur_ses_propres_cles_heritees() {
        let d = TempDir::new().unwrap();
        pose(
            &d,
            "bod.toml",
            &IMPRIMEUR_ESSAI
                .replace(r#"cle = "essai""#, r#"cle = "bod""#)
                .replace(r#"cle_heritee = "essai""#, r#"cle_heritee = "bod""#),
        );
        let (pods, refus) = charge(Some(d.path()));
        assert!(refus.is_empty(), "{refus:?}");
        assert_eq!(pods.len(), 6);
        let bod = pods.iter().find(|p| p.cle == "bod").unwrap();
        assert_eq!(bod.formats[0].nom, "10 × 15 cm");
    }

    /// Un répertoire `pods` qui existe mais ne se lit pas — un fichier ordinaire à sa
    /// place, des droits refusés — n'est pas un poste vierge. Le silence y ferait passer
    /// une avarie pour une absence de surcharges.
    #[test]
    fn un_repertoire_de_surcharges_illisible_est_dit_plutot_qu_ignore() {
        let d = TempDir::new().unwrap();
        std::fs::write(d.path().join("pods"), b"ceci n'est pas un repertoire").unwrap();
        let (pods, refus) = charge(Some(d.path()));
        assert_eq!(refus.len(), 1, "{refus:?}");
        assert!(refus[0].fichier.contains("pods"), "{:?}", refus[0]);
        assert_eq!(pods.len(), 6, "les fournis tiennent malgré tout");
    }

    /// Deux fichiers du poste pour un même POD : le dernier par nom de fichier gagne. Ce
    /// n'est pas dangereux — c'est le même imprimeur — mais ce doit être déterminé :
    /// `read_dir` rend ses entrées dans un ordre que rien ne spécifie.
    #[test]
    fn deux_fichiers_de_meme_cle_pod_se_departagent_par_le_nom_du_fichier() {
        let d = TempDir::new().unwrap();
        pose(
            &d,
            "a-essai.toml",
            &IMPRIMEUR_ESSAI.replace(r#"nom = "Imprimeur d'essai""#, r#"nom = "Premier""#),
        );
        pose(
            &d,
            "z-essai.toml",
            &IMPRIMEUR_ESSAI.replace(r#"nom = "Imprimeur d'essai""#, r#"nom = "Dernier""#),
        );
        let (pods, refus) = charge(Some(d.path()));
        assert!(refus.is_empty(), "{refus:?}");
        assert_eq!(pods.len(), 7, "un seul POD pour deux fichiers");
        let essai = pods.iter().find(|p| p.cle == "essai").unwrap();
        assert_eq!(essai.nom, "Dernier");
    }

    /// Un fichier qui n'est pas un `.toml` n'est pas du catalogue : ni chargé, ni refusé.
    /// Le silence est délibéré — un poste range ce qu'il veut à côté de ses fichiers.
    #[test]
    fn un_fichier_qui_n_est_pas_un_toml_est_ignore_sans_bruit() {
        let d = TempDir::new().unwrap();
        pose(&d, "notes.txt", "ceci n'est pas un POD");
        let (pods, refus) = charge(Some(d.path()));
        assert!(refus.is_empty(), "{refus:?}");
        assert_eq!(pods.len(), 6);
    }

    /// Répertoire de configuration inatteignable : les fournis restent servis, comme les
    /// maquettes fournies le sont quand le poste n'a pas de configuration.
    #[test]
    fn sans_repertoire_de_configuration_les_fournis_restent() {
        let (pods, refus) = charge(None);
        assert!(refus.is_empty());
        assert_eq!(pods.len(), 6);
    }

    /// Le catalogue se sert derrière une référence `'static`, comme la table le faisait :
    /// c'est ce qui garde valides les deux signatures de `commands` qui l'exigent.
    #[test]
    fn un_provider_se_retrouve_par_sa_cle() {
        let pr = provider("bod").expect("bod absent du catalogue");
        assert_eq!(pr.format, (135.0, 215.0));
        assert_eq!(pr.papier_defaut().cle, "creme-90");
        assert!(provider("imprimeur-imaginaire").is_none());
    }

    /// La liste que la Livraison donne à lire, telle qu'elle s'y lit : quatorze entrées,
    /// leurs libellés, leur ordre.
    ///
    /// Le compte, seul, était déjà tenu ; les clés le sont par les ancrages de dos qui les
    /// nomment. Ni l'un ni les autres ne voient un `nom` de POD réécrit ni deux fichiers
    /// fournis permutés — les deux passaient toute la suite. Or c'est un choix de
    /// prestataire que l'utilisateur fait dans cette liste, à la lecture de ces libellés :
    /// l'ordre vient des `FOURNIS`, le libellé du POD et du format, et rien d'autre ici ne
    /// le dit.
    #[test]
    fn la_liste_des_prestataires_garde_ses_quatorze_libelles_dans_l_ordre() {
        let vue: Vec<(&str, &str)> = providers()
            .iter()
            .map(|p| (p.cle.as_str(), p.libelle.as_str()))
            .collect();
        assert_eq!(
            vue,
            [
                ("lulu", "Lulu — poche 108 × 175"),
                ("bod", "BoD — 13,5 × 21,5 cm"),
                ("kdp-5x8", "Amazon KDP — 5 × 8 po"),
                ("kdp-55x85", "Amazon KDP — 5,5 × 8,5 po"),
                ("kdp-6x9", "Amazon KDP — 6 × 9 po"),
                ("coollibri-110x170", "CoolLibri — 11 × 17 cm"),
                ("coollibri-148x210", "CoolLibri — A5"),
                ("coollibri-160x240", "CoolLibri — 16 × 24 cm"),
                ("tbe-110x170", "TheBookEdition — Poche 11 × 17"),
                ("tbe-120x180", "TheBookEdition — Manga 12 × 18"),
                ("tbe-1485x210", "TheBookEdition — A5 14,8 × 21"),
                ("bookvault-127x203", "Bookvault — Novel 127 × 203"),
                ("bookvault-129x198", "Bookvault — B Format 129 × 198"),
                ("bookvault-148x210", "Bookvault — A5 148 × 210"),
            ]
        );
    }

    // Les douze tests qui suivent viennent de la table écrite en dur, dont ils ont suivi
    // la suppression : ils n'ont jamais comparé la vue plate à la table, ils ancrent ses
    // valeurs sur des relevés extérieurs — guides, calculateurs, un livre réel tenu en
    // main. La table partie, ils sont ce qui dit encore que les fichiers fournis portent
    // les bons chiffres. Aucune valeur n'a été recalculée à la migration.

    fn p(cle: &str) -> &'static Provider {
        provider(cle).unwrap_or_else(|| panic!("prestataire inconnu : {cle}"))
    }

    /// Le dos est ce que l'app promet à l'imprimeur : chaque formule est ancrée sur
    /// un relevé réel, pas sur sa propre arithmétique. Si l'un de ces chiffres bouge,
    /// c'est le guide du prestataire qui a changé — pas un détail d'implémentation.
    #[test]
    fn dos_lulu_ancre_sur_le_livre_reel_de_244_pages() {
        let dos = p("lulu").papier_defaut().dos.mm(244).unwrap();
        assert!(
            (dos - 15.48).abs() < 0.01,
            "244 pages → {dos} mm, attendu 15,48"
        );
    }

    #[test]
    fn dos_bod_ancre_sur_le_calculateur_officiel() {
        let d = p("bod").papier_defaut().dos;
        assert!((d.mm(280).unwrap() - 19.5).abs() < 0.05);
        assert!((d.mm(560).unwrap() - 38.4).abs() < 0.05);
    }

    #[test]
    fn dos_kdp_depend_du_papier_et_seulement_du_papier() {
        let kdp = p("kdp-6x9");
        let creme = kdp.papier("creme").unwrap().dos.mm(280).unwrap();
        let blanc = kdp.papier("blanc").unwrap().dos.mm(280).unwrap();
        assert!((creme - 17.78).abs() < 0.01, "crème → {creme} mm");
        assert!((blanc - 16.02).abs() < 0.01, "blanc → {blanc} mm");
        // Le papier ne change que le dos : les trois formats KDP partagent la même
        // composition d'intérieur, donc la même pagination.
        for f in ["kdp-5x8", "kdp-55x85", "kdp-6x9"] {
            assert_eq!(p(f).gouttieres, kdp.gouttieres);
        }
    }

    /// Chez TheBookEdition, le dos ne dépend que de la pagination : leur générateur rend
    /// le même gabarit sur les deux papiers et sur les quatre formats mesurés. Faire
    /// dépendre le dos du papier ici produirait une planche que leur gabarit refuse.
    #[test]
    fn dos_tbe_ancre_sur_les_gabarits_releves() {
        let poche = p("tbe-110x170");
        for (pages, attendu) in [(40, 2.4), (280, 16.8), (750, 45.0)] {
            let dos = poche.papier_defaut().dos.mm(pages).unwrap();
            assert!((dos - attendu).abs() < 0.05, "{pages} p → {dos} mm");
        }
        for papier in &poche.papiers {
            assert_eq!(papier.dos.mm(280), poche.papier_defaut().dos.mm(280));
        }
        for cle in ["tbe-120x180", "tbe-1485x210"] {
            assert_eq!(
                p(cle).papier_defaut().dos.mm(280),
                poche.papier_defaut().dos.mm(280)
            );
        }
    }

    /// Bookvault, à l'inverse, module le dos par le papier : le crème premium fait un
    /// livre visiblement plus épais que le bond blanc à pagination égale.
    #[test]
    fn dos_bookvault_ancre_sur_le_calculateur_papier_par_papier() {
        let bv = p("bookvault-127x203");
        for (cle, pages, attendu) in [
            ("creme-70", 280, 15.7),
            ("creme-70", 800, 44.8),
            ("bond-80", 100, 5.5),
            ("creme-premium-80", 400, 28.8),
        ] {
            let dos = bv.papier(cle).unwrap().dos.mm(pages).unwrap();
            assert!((dos - attendu).abs() < 0.05, "{cle} à {pages} p → {dos} mm");
        }
    }

    /// Le fond perdu est ce qui sépare une planche imprimable d'une planche rejetée.
    /// Chaque valeur vient du gabarit du prestataire, aucune n'est un défaut commun.
    #[test]
    fn le_fond_perdu_est_celui_du_gabarit_de_chaque_prestataire() {
        assert_eq!(p("tbe-110x170").fond_perdu, Some(5.0));
        assert_eq!(p("bookvault-127x203").fond_perdu, Some(3.0));
        assert_eq!(p("coollibri-110x170").fond_perdu, None);
    }

    /// La gouttière se lit dans la tranche, elle ne s'interpole pas : une page de plus
    /// peut la faire basculer, et c'est précisément ce qui oblige à recomposer.
    #[test]
    fn la_gouttiere_bascule_a_la_frontiere_de_tranche() {
        let kdp = p("kdp-6x9");
        assert_eq!(kdp.gouttiere(700).unwrap(), 19.05);
        assert_eq!(kdp.gouttiere(701).unwrap(), 22.23);
    }

    /// Hors tranche connue, on refuse. Inventer une gouttière produirait un intérieur
    /// que le prestataire rejetterait sans que rien ne l'ait signalé.
    #[test]
    fn hors_tranche_le_gabarit_refuse_au_lieu_d_inventer() {
        let err = p("lulu").gouttiere(100).unwrap_err();
        assert!(err.contains("100 pages"), "message peu explicite : {err}");
        assert!(err.contains("lulu"));
    }

    /// Les prestataires à gabarit ne publient pas de formule : l'app ne doit pas
    /// pouvoir en fabriquer une, quelle que soit la pagination.
    #[test]
    fn un_prestataire_a_gabarit_ne_calcule_jamais_de_dos() {
        let cl = p("coollibri-148x210");
        assert_eq!(cl.papier_defaut().dos.mm(280), None);
        assert_eq!(cl.papier_defaut().dos.mm(9999), None);
        assert_eq!(cl.fond_perdu, None, "le fond perdu se relève aussi");
    }

    /// Ce que `verifie` contrôle sur la forme de n'importe quel fichier, celui-ci le
    /// contrôle sur les valeurs des six fournis : l'un dit ce qu'un TOML a le droit
    /// d'écrire, l'autre ce que les nôtres écrivent. Ils ne se remplacent pas.
    #[test]
    fn chaque_prestataire_a_un_papier_par_defaut_et_des_bornes_coherentes() {
        for pr in providers() {
            assert!(!pr.papiers.is_empty(), "{} sans papier", pr.cle);
            assert!(pr.pages_min < pr.pages_max, "{} : bornes inversées", pr.cle);
            assert!(!pr.gouttieres.is_empty(), "{} sans tranche", pr.cle);
            for (lo, hi, g) in &pr.gouttieres {
                assert!(lo <= hi, "{} : tranche inversée", pr.cle);
                assert!(*g > 0.0, "{} : gouttière nulle", pr.cle);
            }
        }
    }

    /// La largeur utile doit rester positive : une gouttière plus large que le format
    /// donnerait une colonne de texte négative, et Typst composerait n'importe quoi.
    ///
    /// `verifie` refuse la colonne nulle de n'importe quel fichier ; celui-ci exige des
    /// six fournis les trente millimètres au-dessous desquels un livre ne se lit plus.
    #[test]
    fn la_colonne_de_texte_reste_positive_sur_toute_la_pagination() {
        for pr in providers() {
            for (lo, _, g) in &pr.gouttieres {
                let utile = pr.format.0 - g - pr.exterieur;
                assert!(
                    utile > 30.0,
                    "{} à {lo} pages : colonne de {utile} mm",
                    pr.cle
                );
            }
        }
    }

    /// Chaque papier dit sa couleur, en notation CSS : c'est le front qui la peint, et
    /// une conversion en chemin serait une occasion de se tromper. La valeur est une
    /// convention d'Ozalid, pas une mesure — aucun prestataire ne publie la teinte de
    /// son crème.
    ///
    /// `verifie` se contente d'une teinte non vide, parce qu'un fichier a le droit
    /// d'écrire n'importe quelle notation CSS ; les six fournis, eux, s'en tiennent tous
    /// à `#rrggbb`, et c'est ce que celui-ci tient.
    #[test]
    fn chaque_papier_annonce_sa_teinte() {
        for p in providers() {
            for pa in &p.papiers {
                assert!(
                    pa.teinte.len() == 7 && pa.teinte.starts_with('#'),
                    "{} / {} : teinte « {} » illisible en CSS",
                    p.cle,
                    pa.cle,
                    pa.teinte
                );
            }
        }
    }

    /* ---------- l'identité à quatre axes ---------- */

    fn fabrication(pod: &str, format: &str, reliure: &str, papier: &str) -> Fabrication {
        Fabrication {
            pod: pod.into(),
            format: format.into(),
            reliure: reliure.into(),
            papier: papier.into(),
        }
    }

    fn pod_de(cle: &str) -> &'static Pod {
        // `pod` est ombré ici par le helper de fabrique de TOML du même nom, `fn pod(blocs:
        // &[&str])` (plus haut dans ce module) : qualifié pour viser sans ambiguïté celui du
        // module.
        super::pod(cle).unwrap_or_else(|| panic!("POD inconnu : {cle}"))
    }

    #[test]
    fn la_cle_d_un_livrable_joint_les_quatre_axes_sans_les_transformer() {
        let f = fabrication("bod", "135x215", "broche", "creme-90");
        // Décision du 26/08 : cinq segments visibles, aucune transformation — le tiret de
        // `creme-90` reste. Une clé se fabrique et se compare, elle ne se découpe jamais.
        assert_eq!(f.cle(), "bod-135x215-broche-creme-90");
        assert_eq!(f.cle_gabarit(), "bod-135x215-broche");
    }

    #[test]
    fn resoudre_un_livrable_rend_les_quatre_references() {
        let r = resout(&fabrication("bod", "135x215", "broche", "creme-90")).unwrap();
        assert_eq!(r.pod.cle, "bod");
        assert_eq!(r.format.cle, "135x215");
        assert_eq!(r.reliure.cle, "broche");
        assert_eq!(r.papier.cle, "creme-90");
        // Le fond perdu du format à défaut, celui du POD sinon : BoD le publie au POD.
        assert_eq!(r.fond_perdu(), Some(5.0));
    }

    #[test]
    fn un_pod_inconnu_est_refuse_en_le_nommant() {
        let e = resout(&fabrication("imaginaire", "135x215", "broche", "creme-90")).unwrap_err();
        assert!(e.contains("imaginaire"), "{e}");
    }

    #[test]
    fn un_format_etranger_au_pod_est_refuse_en_nommant_les_deux() {
        let e = resout(&fabrication("bod", "108x175", "broche", "creme-90")).unwrap_err();
        assert!(e.contains("BoD") && e.contains("108x175"), "{e}");
    }

    #[test]
    fn un_papier_etranger_au_pod_est_refuse() {
        let e = resout(&fabrication("bod", "135x215", "broche", "standard")).unwrap_err();
        assert!(e.contains("standard"), "{e}");
    }

    /// Spec § 9 : une reliure non outillée ne peut pas être choisie, par le Rust, même si
    /// l'interface offrait le contrôle. Le refus porte la raison écrite dans le fichier.
    #[test]
    fn une_reliure_non_outillee_est_refusee_avec_sa_raison() {
        let e = resout(&fabrication("bod", "135x215", "rigide", "creme-90")).unwrap_err();
        assert!(e.contains("rigide"), "{e}");
        assert!(
            e.contains("non relevée"),
            "la raison du fichier doit traverser : {e}"
        );
    }

    /// L'ancrage de la fabrique : pour chacune des quatorze clés héritées, le `Provider`
    /// fabriqué depuis le triplet (et le papier par défaut du POD) est **identique** à celui
    /// de la vue plate — la clé exceptée, neutralisée dans la comparaison, qui change de
    /// convention à la tâche 5. La comparaison porte sur `Provider` entier : un champ ajouté
    /// et renseigné d'un seul côté devient rouge tout seul, sans qu'il faille l'énumérer ici.
    #[test]
    fn le_livrable_resolu_fabrique_le_provider_de_la_vue_plate() {
        for (heritee, pod, format, reliure) in HERITEES {
            let plat = provider(heritee).unwrap_or_else(|| panic!("clé plate absente : {heritee}"));
            let papier = &pod_de(pod).papiers[0].cle;
            let fait = resout(&fabrication(pod, format, reliure, papier))
                .unwrap()
                .provider();
            assert_eq!(
                fait,
                Provider {
                    cle: fait.cle.clone(),
                    ..plat.clone()
                }
            );
        }
    }

    /// La table de migration est ancrée sur les fichiers : chaque ligne désigne un triplet
    /// qui se résout, et la clé héritée qu'elle remplace est bien celle que le format porte
    /// encore. La seconde moitié tombe à la tâche 7 avec `cle_heritee`.
    #[test]
    fn chaque_cle_heritee_a_son_triplet() {
        for (heritee, pod, format, reliure) in HERITEES {
            let r = resout(&Fabrication {
                pod: pod.into(),
                format: format.into(),
                reliure: reliure.into(),
                papier: pod_de(pod).papiers[0].cle.clone(),
            })
            .unwrap_or_else(|e| panic!("{heritee} : {e}"));
            assert_eq!(r.format.cle_heritee, heritee);
        }
    }

    /// L'empreinte ne voit que ce qui pagine : le format, les marges, les gouttières.
    /// Ni le papier, ni le fond perdu, ni la formule de dos — le dos affiché se recalcule,
    /// lui, à chaque vue.
    #[test]
    fn l_empreinte_ne_bouge_qu_avec_ce_qui_pagine() {
        let r = resout(&fabrication("bod", "135x215", "broche", "creme-90")).unwrap();
        assert_eq!(r.empreinte(), "135x215|18.8|28|15|24-900-20");
    }
}
