//! Le catalogue des POD : ce que chaque imprimeur offre, et d'où chaque chiffre vient.
//!
//! Un fichier TOML par POD. Les fournis sont incorporés au binaire par `include_str!` —
//! il n'y a donc aucun chemin à résoudre pour eux, aucun mode dégradé, aucun écart entre
//! développement et livraison. Le poste peut en déposer d'autres, qui remplacent le
//! fourni de même clé.
//!
//! Cinq axes : le POD, ses formats, ses reliures, ses finitions, ses papiers. Le cas
//! courant — tout compatible avec tout — ne s'écrit pas ; seules les exceptions se
//! déclarent. Un arbre POD > format > reliure > papier aurait obligé à recopier les
//! quatre papiers d'un POD sous chacun de ses formats.
//!
//! Règle qui tient tout le reste : **une valeur qu'on n'a pas lue ne s'écrit pas**, et
//! une valeur d'énumération que le code ne sait pas appliquer est refusée plutôt
//! qu'ignorée. Le fichier de données ne doit pas pouvoir promettre plus que le code.

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
pub struct Marges {
    pub haut: f64,
    pub bas: f64,
    /// Marge extérieure (sécurité), opposée à la gouttière.
    pub exterieur: f64,
}

/// Une tranche de pagination et la gouttière (marge intérieure) qu'elle impose.
pub type Tranche = (u32, u32, f64);

#[derive(Debug, Clone, Deserialize)]
pub struct Format {
    pub cle: String,
    pub nom: String,
    /// **Transitoire.** La clé plate que portent encore le `.ozalid`, les répertoires de
    /// package et l'interface. Elle disparaît au lot 2, avec la migration des projets.
    pub cle_heritee: String,
    /// Format de rognage en mm (largeur, hauteur).
    pub mm: (f64, f64),
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
    pub pages: Option<(u32, u32)>,
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
pub struct Finition {
    pub cle: String,
    pub nom: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
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

impl Pod {
    /// Lit un POD depuis son TOML, et le refuse s'il promet ce que le code ne tient pas.
    pub fn depuis_toml(s: &str) -> Result<Self, String> {
        let pod: Pod = toml::from_str(s).map_err(|e| e.to_string())?;
        pod.verifie()?;
        Ok(pod)
    }

    fn verifie(&self) -> Result<(), String> {
        for r in &self.reliures {
            match (&r.geometrie, &r.non_outille) {
                (None, None) => {
                    return Err(format!(
                        "{} / {} : ni géométrie ni raison de ne pas en avoir. Une reliure \
                         qu'on n'outille pas doit dire pourquoi.",
                        self.cle, r.cle
                    ))
                }
                (Some(_), _) if r.pages.is_none() || r.parite.is_none() => {
                    return Err(format!(
                        "{} / {} : une reliure outillée doit porter sa pagination admise \
                         et sa parité.",
                        self.cle, r.cle
                    ))
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
mm = [135.0, 215.0]
marges = { haut = 18.8, bas = 28.0, exterieur = 15.0 }
gouttieres = [[24, 900, 20.0]]

[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = [24, 900]
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
        assert_eq!(pod.formats[0].mm, (135.0, 215.0));
        assert_eq!(pod.formats[0].marges.bas, 28.0);
        assert_eq!(pod.formats[0].gouttieres, vec![(24, 900, 20.0)]);
        assert_eq!(pod.reliures[0].geometrie, Some(Geometrie::DosCarreColle));
        assert_eq!(pod.reliures[0].pages, Some((24, 900)));
        assert_eq!(pod.papiers[0].teinte, "#f7f0e0");
        // Comparaison à la tolérance : 0,0675 n'a pas de représentation binaire
        // exacte, et `280 × 0,0675 + 0,6` ne vaut pas `19.5` au bit près.
        let dos = pod.papiers[0].dos.mm(280).unwrap();
        assert!((dos - 19.5).abs() < 1e-9, "dos {dos}");
    }

    /// Une géométrie que le code ne sait pas appliquer est **refusée**, jamais ignorée.
    /// C'est ce qui empêche un fichier d'annoncer une reliure que la planche ne compose
    /// pas : le fichier de données ne doit pas pouvoir promettre plus que le code.
    #[test]
    fn une_geometrie_inconnue_est_refusee_en_la_nommant() {
        let e = Pod::depuis_toml(
            r#"
cle = "essai"
nom = "Imprimeur d'essai"

[[reliure]]
cle = "cousu"
nom = "Reliure cousue"
geometrie = "cousue"
pages = [24, 900]
parite = "paire"
"#,
        )
        .unwrap_err();
        assert!(e.contains("cousue"), "{e}");
    }

    /// Une reliure qui n'annonce ni géométrie ni raison de ne pas en avoir est un oubli,
    /// pas un choix : on ne peut pas deviner si elle est composable.
    #[test]
    fn une_reliure_sans_geometrie_ni_raison_est_refusee() {
        let e = Pod::depuis_toml(
            r#"
cle = "essai"
nom = "Imprimeur d'essai"

[[reliure]]
cle = "rigide"
nom = "Couverture rigide"
"#,
        )
        .unwrap_err();
        assert!(e.contains("rigide"), "{e}");
    }

    /// Une reliure outillée sans pagination admise laisserait `package` accepter
    /// n'importe quel compte de pages : le refus de pagination est un contrôle, pas une
    /// décoration.
    #[test]
    fn une_reliure_outillee_sans_pagination_est_refusee() {
        let e = Pod::depuis_toml(
            r#"
cle = "essai"
nom = "Imprimeur d'essai"

[[reliure]]
cle = "broche"
nom = "Broché"
geometrie = "dos-carre-colle"
parite = "paire"
"#,
        )
        .unwrap_err();
        assert!(e.contains("broche"), "{e}");
    }
}
