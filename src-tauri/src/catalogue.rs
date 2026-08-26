//! Le catalogue des POD : ce que chaque imprimeur offre, et d'où chaque chiffre vient.
//!
//! Un fichier TOML par POD. Les fournis seront incorporés au binaire par `include_str!` —
//! il n'y aura donc aucun chemin à résoudre pour eux, aucun mode dégradé, aucun écart
//! entre développement et livraison. Le poste pourra en déposer d'autres, qui
//! remplaceront le fourni de même clé. Ce module ne porte pour l'instant que les types et
//! leur lecture : ni fichier fourni, ni chargement, ni vue plate.
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

use serde::Deserialize;

/// Épaisseur du dos. Trois formes, parce que les prestataires en publient trois.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(tag = "forme", rename_all = "lowercase")]
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
        Ok(())
    }

    fn verifie_format(&self, f: &Format) -> Result<(), String> {
        let ou = format!("{} / {}", self.cle, f.cle);
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
        for t in &f.gouttieres {
            if t.de > t.a {
                return Err(format!(
                    "{ou} : tranche de gouttière à l'envers ({}–{} pages).",
                    t.de, t.a
                ));
            }
            if !fini_non_negatif(t.mm) {
                return Err(format!("{ou} : gouttière impossible ({} mm).", t.mm));
            }
        }
        if let Some(fp) = f.fond_perdu {
            if !fini_non_negatif(fp) {
                return Err(format!("{ou} : fond perdu impossible ({fp} mm)."));
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
            (Some(_), None) if r.pages.is_none() || r.parite.is_none() => {
                return Err(format!(
                    "{ou} : une reliure outillée doit porter sa pagination admise et sa \
                     parité."
                ))
            }
            _ => {}
        }
        if let Some(p) = r.pages {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Le socle d'un POD valide. Les tests de refus n'écrivent que le bloc qu'ils mettent
    // en cause : sans ce socle ils échoueraient sur une liste vide, et leur assertion sur
    // la clé fautive ne dirait plus rien de ce qu'ils testent.
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
cle_heritee = "essai"
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
    }

    /// Une reliure qui porte à la fois sa géométrie et sa raison de ne pas en avoir dit
    /// deux choses contraires : l'appelant qui interroge `geometrie` la compose, celui qui
    /// interroge `non_outille` la grise. C'est au fichier de trancher, pas à eux.
    #[test]
    fn une_reliure_a_la_fois_outillee_et_non_outillee_est_refusee() {
        let r = format!("{RELIURE}non_outille = \"géométrie non relevée\"\n");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("broche"), "{e}");
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

    /// Une reliure outillée sans pagination admise laisserait `package` accepter
    /// n'importe quel compte de pages : le refus de pagination est un contrôle, pas une
    /// décoration.
    #[test]
    fn une_reliure_outillee_sans_pagination_est_refusee() {
        let r = sauf(RELIURE, "pages = { min = 24, max = 900 }\n", "");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("broche"), "{e}");
    }

    /// Même chose pour la parité : sans elle, `interieur` ne sait pas s'il doit ajouter
    /// une blanche de fin, et la composition perd la seule règle qu'elle sache tenir.
    #[test]
    fn une_reliure_outillee_sans_parite_est_refusee() {
        let r = sauf(RELIURE, "parite = \"paire\"\n", "");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("broche"), "{e}");
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
        }
        for plus in ["nan", "-1.0"] {
            let p = sauf(PAPIER, "plus = 0.6", &format!("plus = {plus}"));
            let e = Pod::depuis_toml(&pod(&[FORMAT, RELIURE, &p])).unwrap_err();
            assert!(e.contains("creme-90"), "plus = {plus} : {e}");
        }
    }

    /// Même refus pour la géométrie de la page : un format de rognage nul ou non fini, une
    /// marge négative, un fond perdu impossible traversent tout aussi loin — jusqu'au
    /// gabarit de l'intérieur, où ils ne provoquent pas une erreur mais une page fausse.
    #[test]
    fn une_dimension_ou_une_marge_impossible_est_refusee() {
        for (avant, apres) in [
            ("largeur = 135.0", "largeur = 0.0"),
            ("hauteur = 215.0", "hauteur = nan"),
            ("bas = 28.0", "bas = -28.0"),
            ("mm = 20.0", "mm = inf"),
        ] {
            let f = sauf(FORMAT, avant, apres);
            let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
            assert!(e.contains("135x215"), "{apres} : {e}");
        }
        let e =
            Pod::depuis_toml(&pod(&["fond_perdu = -5.0\n", FORMAT, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("essai"), "{e}");
    }

    /// Une tranche dont la borne basse dépasse la haute n'admet aucune valeur : elle
    /// refuserait toute pagination, ou n'appliquerait jamais sa gouttière. C'est une
    /// coquille de saisie, et elle doit se voir au chargement plutôt qu'à la composition.
    #[test]
    fn une_tranche_a_l_envers_est_refusee() {
        let r = sauf(RELIURE, "min = 24, max = 900", "min = 900, max = 24");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &r, PAPIER])).unwrap_err();
        assert!(e.contains("broche"), "{e}");

        let f = sauf(FORMAT, "de = 24, a = 900", "de = 900, a = 24");
        let e = Pod::depuis_toml(&pod(&[&f, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("135x215"), "{e}");
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
    }

    /// Deux entrées de même clé, et le `find` des appelants en prend une sans que rien ne
    /// dise laquelle — deux papiers homonymes aux dos différents donneraient deux
    /// épaisseurs selon l'appelant. La `cle_heritee` compte double : elle nomme un
    /// répertoire de package.
    #[test]
    fn deux_cles_identiques_sont_refusees() {
        let e = Pod::depuis_toml(&pod(&[FORMAT, RELIURE, PAPIER, PAPIER])).unwrap_err();
        assert!(e.contains("creme-90"), "{e}");

        let autre = sauf(FORMAT, "cle = \"135x215\"", "cle = \"148x210\"");
        let e = Pod::depuis_toml(&pod(&[FORMAT, &autre, RELIURE, PAPIER])).unwrap_err();
        assert!(e.contains("essai-135x215"), "{e}");
    }

    /// `papier_defaut()` indexera `papiers[0]`, et le choix d'un format comme d'une
    /// reliure suppose qu'il en existe au moins un. L'invariant que `&'static [Papier]`
    /// tenait par construction n'a plus que le chargement où vivre : un POD amputé doit
    /// être refusé, pas faire paniquer l'application au premier clic.
    #[test]
    fn un_pod_sans_format_reliure_ou_papier_est_refuse() {
        for blocs in [[RELIURE, PAPIER], [FORMAT, PAPIER], [FORMAT, RELIURE]] {
            let e = Pod::depuis_toml(&pod(&blocs)).unwrap_err();
            assert!(e.contains("essai"), "{blocs:?} : {e}");
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
}
