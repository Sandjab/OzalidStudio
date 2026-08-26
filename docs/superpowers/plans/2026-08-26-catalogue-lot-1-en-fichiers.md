# Catalogue lot 1 — la table en fichiers

> **Pour un exécutant agentique :** SOUS-COMPÉTENCE REQUISE : `superpowers:subagent-driven-development`
> (recommandé) ou `superpowers:executing-plans` pour exécuter ce plan tâche par tâche. Les
> étapes sont des cases à cocher (`- [ ]`).

**But :** remplacer les quatorze entrées écrites en dur de `providers.rs` par six fichiers
TOML — un par POD — lus au lancement, surchargeables depuis le poste, **sans rien changer à
ce que l'application affiche ni à ce qu'elle compose**.

**Architecture :** un module `catalogue` porte les types à cinq axes (POD, format, reliure,
finition, papier) et leur lecture TOML. Les six fichiers fournis sont incorporés au binaire
par `include_str!`, comme les maquettes fournies ; le répertoire de configuration peut en
déposer d'autres, qui remplacent le fourni de même clé. Une **vue plate** dérivée du
catalogue produit les `Provider` que tout le reste du code consomme déjà : c'est ce qui rend
ce lot invisible, et un test transitoire compare cette vue, champ par champ, à la table
actuelle avant de la supprimer.

**Pile :** Rust 2021, Tauri 2, `serde` + `toml 0.8` (déjà dépendances), `tempfile` en
dev-dependency. Front vanilla. Tests : `cargo test` depuis `src-tauri/`,
`node --test tests/*.test.js` depuis la racine, et `cargo run --example temoin` comme témoin
de non-régression.

**Spec :** `docs/superpowers/specs/2026-08-26-catalogue-et-livrables-design.md`.

---

## Avant chaque commit

Ces quatre vérifications valent pour **toutes** les étapes « Commit » de ce plan, sans être
répétées à chaque fois. Depuis `src-tauri/` :

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Depuis la racine :

```
node --test tests/*.test.js
```

Et, tout fichier de `src-tauri/` ayant changé à chaque tâche de ce plan :

```
cd src-tauri && cargo run --example temoin
```

Attendu, à chaque tâche, sans exception : **98 pages, dos 7,21 mm**. C'est la valeur de
`PAGES_ATTENDUES` (`src-tauri/examples/temoin.rs:34`), relevée sur macOS avec Typst 0.15.1 et
EB Garamond. Un écart n'est pas à corriger dans le témoin : c'est le signe que la conversion
a perdu une valeur.

## Structure des fichiers

| Fichier | Responsabilité |
|---|---|
| `src-tauri/src/catalogue.rs` | **Créé.** Les types à cinq axes, leur lecture TOML, leur validation, le chargement (fournis + poste), et la vue plate `Provider` |
| `src-tauri/pods/*.toml` | **Créés.** Six fichiers, un par POD, incorporés par `include_str!` |
| `src-tauri/src/providers.rs` | **Supprimé** à la tâche 4 — ses valeurs **et ses tests d'ancrage** ayant migré |
| `src-tauri/src/lib.rs` | **Modifié.** `pub mod catalogue;` remplace `pub mod providers;` ; `initialiser` appelé dans `.setup()` |
| `src-tauri/src/commands.rs` | **Modifié.** `providers_liste`, la commande de refus, et les deux signatures `&'static Provider` |
| `src-tauri/src/interieur.rs` | **Modifié** à la tâche 7. Corps, interligne et folio deviennent des constantes |
| `src/livraison.js` | **Modifié** à la tâche 6. La ligne qui nomme un fichier de catalogue refusé |
| `tests/contrats.test.js` | **Modifié** à la tâche 6 |

`catalogue.rs` porte tout le domaine du catalogue — types, lecture, validation, chargement,
vue plate — parce que ces cinq choses changent ensemble : ajouter un axe les touche toutes.
C'est le même découpage que `maquettes.rs`, qui porte le format, les fournies, les
personnalisées et le slug dans un seul fichier.

---

### Tâche 1 : Les types du catalogue et leur lecture

**Fichiers :**
- Créer : `src-tauri/src/catalogue.rs`
- Modifier : `src-tauri/src/lib.rs` (ajouter `pub mod catalogue;`)

- [ ] **Étape 1 : Écrire le test qui échoue**

Dans `src-tauri/src/catalogue.rs`, créer le fichier avec **seulement** ce bloc de tests :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Le TOML d'un POD se lit tel qu'il est écrit. Ce test tient la forme du format :
    /// s'il change, tous les fichiers fournis changent avec lui.
    #[test]
    fn un_pod_se_lit_depuis_son_toml() {
        // `r##"…"##` et non `r#"…"#` : la séquence `"#` de `teinte = "#f7f0e0"`
        // fermerait le littéral.
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
gouttieres = [ { de = 24, a = 900, mm = 20.0 } ]

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
        assert_eq!(pod.formats[0].mm.largeur, 135.0);
        assert_eq!(pod.formats[0].mm.hauteur, 215.0);
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
        assert_eq!(pod.reliures[0].pages, Some(Pagination { min: 24, max: 900 }));
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
pages = { min = 24, max = 900 }
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
```

Ajouter `pub mod catalogue;` dans `src-tauri/src/lib.rs`. La liste y est alphabétique :
`catalogue` se pose **avant** `pub mod commands;`, en première ligne du fichier.

- [ ] **Étape 2 : Lancer le test pour le voir échouer**

```
cd src-tauri && cargo test --lib catalogue
```

Attendu : ÉCHEC de compilation, `cannot find type Pod in this scope`.

- [ ] **Étape 3 : Écrire l'implémentation minimale**

Au-dessus du bloc de tests, dans `src-tauri/src/catalogue.rs` :

```rust
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

/// Dimensions d'un format de rognage, en mm.
///
/// Nommées et non positionnelles : ces fichiers s'éditent à la main, des années après
/// avoir été écrits, et une largeur prise pour une hauteur donne un livre à l'italienne
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
pub struct Format {
    pub cle: String,
    pub nom: String,
    /// **Transitoire.** La clé plate que portent encore le `.ozalid`, les répertoires de
    /// package et l'interface. Elle disparaît au lot 2, avec la migration des projets.
    pub cle_heritee: String,
    /// Format de rognage.
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
```

Si `toml` refuse l'énumération étiquetée de l'intérieur (`#[serde(tag = "forme")]`) —
elle demande au désérialiseur de tamponner son contenu, ce que toutes les versions ne font
pas de la même façon —, replier `Dos` sur une structure plate
`{ forme: String, par: Option<f64>, plus: Option<f64> }` et la convertir à la main dans
`verifie`, en refusant une `forme` inconnue. Le TOML écrit ne change pas ; le premier test
de la tâche tranche.

Note sur le message de `une_geometrie_inconnue_est_refusee_en_la_nommant` : c'est `serde` qui
le produit, et il cite la valeur reçue (`unknown variant \`cousue\``). Le test n'exige rien
de plus que la présence du mot fautif — le formuler nous-mêmes obligerait à désérialiser à la
main.

- [ ] **Étape 4 : Lancer les tests pour les voir passer**

```
cd src-tauri && cargo test --lib catalogue
```

Attendu : 4 tests passent.

- [ ] **Étape 5 : Commit**

```
git add src-tauri/src/catalogue.rs src-tauri/src/lib.rs
git commit -m "Le catalogue a ses types, et refuse ce qu'il ne sait pas tenir"
```

---

#### Ce que l'exécution a ajouté à la tâche 1

Écrit après coup, pour que ce plan ne mente pas à qui le relira. La tâche 1 a été livrée
telle que décrite ci-dessus (`27ca0ce`), puis une revue de qualité a montré que `verifie`
ne regardait que le couple géométrie/raison — aucun nombre, aucune clé, aucune liste. Or
c'est le nombre qui va au massicot. Dix corrections ont suivi (`17241e7`, `102c1e3`) :

- **Les nombres sont validés.** `par = nan` était accepté — TOML 1.0 lit littéralement
  `nan` et `inf` —, `forme = "divise", par = 0.0` donnait un dos infini et `par = -0.07`
  un dos de −19,6 mm. Sont refusés : facteur de dos non fini ou ≤ 0, constante non finie
  ou < 0, dimension non finie ou ≤ 0, marge ou fond perdu non fini ou < 0.
- **`#[serde(deny_unknown_fields)]`** sur les structures. Sans lui, `fond-perdu` avec un
  tiret était ignoré sans erreur : quelqu'un relève une valeur, l'écrit, et le catalogue
  fait comme s'il ne l'avait pas. C'est le pire défaut possible dans un module dont la
  règle est qu'on ne reporte que ce qu'on a lu.
- **Une reliure ne peut pas porter à la fois `geometrie` et `non_outille`.** Deux
  appelants qui n'interrogent pas le même champ en tireraient deux réponses opposées.
- **Unicité des `cle`** dans les quatre listes et des `cle_heritee` entre formats ; **au
  moins un format, une reliure, un papier** — `papier_defaut()` indexera `papiers[0]`, et
  l'invariant que `&'static [Papier]` tenait par construction n'existe plus ; **bornes**
  `de <= a` et `min <= max`.
- **Les tuples positionnels deviennent `Dimensions`, `Tranche` et `Pagination`** — c'est
  la forme qui figure dans les blocs de code ci-dessus, mise à jour.
- **Cinq tests neufs** là où rien ne protégeait : parité inconnue, reliure outillée sans
  parité, reliure non outillée lue avec sa raison, `Dos::Divise` sur le cas Lulu vérifié
  sur livre réel (244 pages → 15,48 mm), `Dos::Mesure` rendant `None`.

Un contrôle a été **écarté** et ne doit pas être ajouté plus tard par mégarde : vérifier
que les tranches de gouttière couvrent la pagination admise d'une reliure. Le catalogue
réel ne le respecte pas — Lulu accepte 32 à 800 pages en dos carré collé et ne publie de
gouttière que pour 151 à 400. Le refus tardif à la composition est le comportement voulu,
et le COOKBOOK le documente comme piège.

---

### Tâche 2 : Les six fichiers fournis

**Fichiers :**
- Créer : `src-tauri/pods/lulu.toml`, `bod.toml`, `kdp.toml`, `coollibri.toml`,
  `thebookedition.toml`, `bookvault.toml`
- Modifier : `src-tauri/src/catalogue.rs`

- [ ] **Étape 1 : Écrire le test qui échoue**

Dans le `mod tests` de `catalogue.rs` :

```rust
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
```

- [ ] **Étape 2 : Lancer le test pour le voir échouer**

```
cd src-tauri && cargo test --lib catalogue
```

Attendu : ÉCHEC de compilation, `cannot find function fournis in this scope`.

- [ ] **Étape 3 : Écrire les six fichiers et la fonction**

Dans `catalogue.rs`, sous les types :

```rust
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
```

Puis écrire les six TOML. **La source qui fait foi est `src-tauri/src/providers.rs` dans son
état actuel** : chaque valeur s'y recopie, y compris ses commentaires de provenance, qui
descendent en commentaires TOML et en champs `source`. Ne rien arrondir, ne rien recalculer,
ne rien déduire — la tâche 3 compare la vue plate à la table historique, valeur par valeur,
et c'est ce test qui rattrape une transcription fautive.

Correspondance des quatorze entrées historiques :

| Fichier | `cle` du POD | Formats (`cle` → `cle_heritee`) |
|---|---|---|
| `lulu.toml` | `lulu` | `108x175` → `lulu` |
| `bod.toml` | `bod` | `135x215` → `bod` |
| `kdp.toml` | `kdp` | `5x8` → `kdp-5x8`, `55x85` → `kdp-55x85`, `6x9` → `kdp-6x9` |
| `coollibri.toml` | `coollibri` | `110x170` → `coollibri-110x170`, `148x210` → `coollibri-148x210`, `160x240` → `coollibri-160x240` |
| `thebookedition.toml` | `tbe` | `110x170` → `tbe-110x170`, `120x180` → `tbe-120x180`, `1485x210` → `tbe-1485x210` |
| `bookvault.toml` | `bookvault` | `127x203` → `bookvault-127x203`, `129x198` → `bookvault-129x198`, `148x210` → `bookvault-148x210` |

Le `nom` du POD est son nom d'imprimeur seul, le `nom` du format son format seul
(« 13,5 × 21,5 cm », « 5,5 × 8,5 po ») : le `libelle` plat d'aujourd'hui — « Amazon KDP —
5,5 × 8,5 po » — se reconstitue à la tâche 3 en les joignant par « — ». C'est ce qui garde
l'interface identique.

**Contrainte qui en découle, et qui prime sur l'envie de bien nommer : le `nom` du POD doit
être exactement le préfixe du libellé historique.** Donc `nom = "BoD"`, et non
« BoD (Books on Demand) » : ce dernier donnerait « BoD (Books on Demand) — 13,5 × 21,5 cm »
à l'écran, ferait échouer la tâche 3, et changerait l'interface — contre le but déclaré du
lot. Le nom complet vit en commentaire de tête du fichier. Les cinq autres n'ont pas ce
problème : « Lulu », « Amazon KDP », « CoolLibri », « TheBookEdition » et « Bookvault » sont
déjà les préfixes historiques.

Reliures et finitions, pour les six : chacun porte au minimum

```toml
[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = { min = 24, max = 900 }   # les bornes de son entrée historique
parite = "paire"
```

où `pages` reprend `pages_min` et `pages_max` de l'entrée historique correspondante. Aucune
autre reliure, aucune finition n'est écrite dans ce lot : les ajouter demanderait des relevés
qui n'ont pas été faits, et la règle du dépôt est de ne reporter que ce qu'on a lu. Le lot 4
les apporte pour BoD.

`bod.toml` en entier, comme gabarit d'écriture pour les cinq autres :

```toml
# BoD (Books on Demand) — Hambourg, filiale française, impression Europe.
#
# Imprimer n'oblige pas à publier : le parcours myBoD permet de commander pour soi sans
# référencer le titre. C'est ce qui en fait le défaut du comparatif POD du 19/08/2026.

cle = "bod"
# « BoD » seul : c'est le préfixe du libellé historique, et la tâche 3 le vérifie.
nom = "BoD"
# Guide de maquette BoD. Commun à ses formats.
fond_perdu = 5.0

[[format]]
cle = "135x215"
nom = "13,5 × 21,5 cm"
cle_heritee = "bod"
mm = { largeur = 135.0, hauteur = 215.0 }
marges = { haut = 18.8, bas = 28.0, exterieur = 15.0 }
# BoD ne module pas la marge de reliure selon l'épaisseur — tranche unique, couvrant les
# 24 à 900 pages que sa couverture souple admet.
gouttieres = [ { de = 24, a = 900, mm = 20.0 } ]
source = "modèle Word « Roman » 13,5 × 21,5"

[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = { min = 24, max = 900 }
parite = "paire"
source = "validation du calculateur officiel : nombre pair obligatoire"

# BoD : dos = pages × épaisseur_feuille/2 + 0,6 mm de couverture 250 g. L'épaisseur dépend
# du papier ; retenu le crème 90 g, défaut de BoD et papier de roman.
[[papier]]
cle = "creme-90"
nom = "Crème 90 g"
teinte = "#f7f0e0"
dos = { forme = "multiplie", par = 0.0675, plus = 0.6 }
source = "calculateur officiel, relevé sur 4 points — 280 p → 19,5 mm, 560 p → 38,4 mm"
```

- [ ] **Étape 4 : Lancer les tests pour les voir passer**

```
cd src-tauri && cargo test --lib catalogue
```

Attendu : 6 tests passent, dont `les_six_fichiers_fournis_se_lisent` (6 POD, 14 formats).

- [ ] **Étape 5 : Commit**

```
git add src-tauri/pods src-tauri/src/catalogue.rs
git commit -m "Les six POD ont leur fichier, et le binaire les embarque"
```

---

### Tâche 3 : La vue plate, et la preuve qu'elle ne change rien

C'est la tâche qui rend le lot invisible. La table historique est **conservée** le temps
d'un test qui la compare à la vue dérivée, puis supprimée à la tâche 4.

**Fichiers :**
- Modifier : `src-tauri/src/catalogue.rs`
- Modifier : `src-tauri/src/providers.rs` (renommer la constante)

- [ ] **Étape 1 : Écrire le test qui échoue**

Dans `src-tauri/src/providers.rs`, renommer la constante publique :

```rust
/// **Transitoire.** La table historique, gardée le temps de prouver que le catalogue lu
/// depuis les TOML rend exactement ce qu'elle rendait. Supprimée à la tâche 4 du lot 1.
pub const PROVIDERS_HERITEE: &[Provider] = &[
```

et corriger **tous** ses usages. Ils ne sont pas un seul, contrairement à ce que ce plan a
d'abord écrit : `commands.rs:156`, `projet.rs:325` (le `impl Default for Livraison`) et
`examples/composer.rs:25` (la liste des prestataires du message d'usage) hors tests, plus
`maquettes.rs:923` et `projet.rs:1356` dans des tests, plus quatre internes à `providers.rs`.
Un renommage partiel ne compile pas. La seule occurrence à **laisser** est `providers.rs:4`,
qui parle du `PROVIDERS` d'`index.html` — une autre variable, dans un autre fichier.

Puis, dans le `mod tests` de `catalogue.rs` :

```rust
/// La vue plate rend, valeur par valeur, ce que la table écrite en dur rendait.
///
/// Test **transitoire** : il meurt avec la table, à la tâche 4. C'est la seule preuve que
/// la conversion des quatorze entrées n'a pas perdu un dixième de millimètre — et une
/// valeur fausse ne se verrait autrement qu'au massicot.
#[test]
fn la_vue_plate_rend_ce_que_la_table_historique_rendait() {
    let vue = plats().unwrap();
    let heritee = crate::providers::PROVIDERS_HERITEE;
    assert_eq!(vue.len(), heritee.len());
    for (v, h) in vue.iter().zip(heritee) {
        assert_eq!(v.cle, h.cle, "clé");
        assert_eq!(v.libelle, h.libelle, "{} : libellé", h.cle);
        assert_eq!(v.format, h.format, "{} : format", h.cle);
        assert_eq!(v.marge_haut, h.marge_haut, "{} : marge haut", h.cle);
        assert_eq!(v.marge_bas, h.marge_bas, "{} : marge bas", h.cle);
        assert_eq!(v.exterieur, h.exterieur, "{} : extérieur", h.cle);
        assert_eq!(v.gouttieres.as_slice(), h.gouttieres, "{} : gouttières", h.cle);
        assert_eq!(v.corps_pt, h.corps_pt, "{} : corps", h.cle);
        assert_eq!(v.interligne, h.interligne, "{} : interligne", h.cle);
        assert_eq!(v.folio_pt, h.folio_pt, "{} : folio", h.cle);
        assert_eq!(v.fond_perdu, h.fond_perdu, "{} : fond perdu", h.cle);
        assert_eq!(v.pages_min, h.pages_min, "{} : pages min", h.cle);
        assert_eq!(v.pages_max, h.pages_max, "{} : pages max", h.cle);
        assert_eq!(v.papiers.len(), h.papiers.len(), "{} : papiers", h.cle);
        for (pv, ph) in v.papiers.iter().zip(h.papiers) {
            assert_eq!(pv.cle, ph.cle, "{} : clé papier", h.cle);
            // Le catalogue dit `nom` là où la table disait `libelle` : c'est le même
            // fait, et le nom du champ suit celui du POD et du format.
            assert_eq!(pv.nom, ph.libelle, "{} : nom du papier", h.cle);
            assert_eq!(pv.teinte, ph.teinte, "{} : teinte", h.cle);
            assert_eq!(pv.dos.mm(280), ph.dos.mm(280), "{} : dos à 280 p", h.cle);
        }
    }
}
```

- [ ] **Étape 2 : Lancer le test pour le voir échouer**

```
cd src-tauri && cargo test --lib catalogue
```

Attendu : ÉCHEC de compilation, `cannot find function plats in this scope`.

- [ ] **Étape 3 : Écrire l'implémentation minimale**

Dans `catalogue.rs`, la structure plate et sa dérivation. Elle reprend le `Provider`
historique en remplaçant les `&'static str` par des `String` : la liste vit désormais dans un
`OnceLock`, ce qui rend les `&'static Provider` toujours obtenables.

```rust
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
    /// Triplets, comme la table historique les écrivait : la vue plate est comparée à
    /// elle, champ par champ, à la tâche 3.
    pub gouttieres: Vec<(u32, u32, f64)>,
    pub corps_pt: f64,
    pub interligne: f64,
    pub folio_pt: f64,
    pub fond_perdu: Option<f64>,
    pub pages_min: u32,
    pub pages_max: u32,
    pub papiers: Vec<Papier>,
}

/// Corps, interligne et folio de l'intérieur.
///
/// Ils étaient dans les quatorze entrées de la table, **identiques dans toutes**. Ce ne
/// sont pas des faits de prestataire mais des réglages typographiques : ils quittent le
/// catalogue à la tâche 7, où ils deviennent les constantes de `interieur`. Ils sont
/// reproduits ici le temps que la vue plate porte encore ces champs.
const CORPS_PT: f64 = 9.5;
const INTERLIGNE: f64 = 1.42;
const FOLIO_PT: f64 = 8.0;

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
pub fn aplatit(pods: &[Pod]) -> Vec<Provider> {
    let mut v = Vec::new();
    for pod in pods {
        let Some(r) = pod.reliures.iter().find(|r| r.geometrie.is_some()) else {
            continue;
        };
        let Some(pagination) = r.pages else {
            continue;
        };
        for f in &pod.formats {
            v.push(Provider {
                cle: f.cle_heritee.clone(),
                libelle: format!("{} — {}", pod.nom, f.nom),
                // La vue plate garde les tuples de la table historique : c'est ce qui
                // permet au test de non-régression de comparer sans traduction.
                format: (f.mm.largeur, f.mm.hauteur),
                marge_haut: f.marges.haut,
                marge_bas: f.marges.bas,
                exterieur: f.marges.exterieur,
                gouttieres: f.gouttieres.iter().map(|t| (t.de, t.a, t.mm)).collect(),
                corps_pt: CORPS_PT,
                interligne: INTERLIGNE,
                folio_pt: FOLIO_PT,
                fond_perdu: f.fond_perdu.or(pod.fond_perdu),
                pages_min: pagination.min,
                pages_max: pagination.max,
                papiers: pod.papiers.clone(),
            });
        }
    }
    v
}

/// La vue plate des fournis.
pub fn plats() -> Result<Vec<Provider>, String> {
    Ok(aplatit(&fournis()?))
}
```

**Le libellé.** `format!("{} — {}", pod.nom, f.nom)` doit rendre exactement les quatorze
libellés d'aujourd'hui — « Lulu — poche 108 × 175 », « BoD — 13,5 × 21,5 cm », « Amazon KDP —
5 × 8 po », « CoolLibri — 11 × 17 cm », « TheBookEdition — Poche 11 × 17 », « Bookvault —
Novel 127 × 203 ». C'est le test qui l'impose ; ajuster `nom` dans les TOML jusqu'à ce qu'il
passe, jamais l'inverse.

- [ ] **Étape 4 : Lancer le test pour le voir passer**

```
cd src-tauri && cargo test --lib catalogue
```

Attendu : tous les tests de `catalogue` passent — ils sont vingt-cinq à ce stade, la tâche 1
en ayant apporté bien plus que ce plan ne le prévoyait. Un échec nomme la clé et le champ
fautifs : c'est une valeur mal recopiée dans un TOML, à corriger là et non dans le test.

- [ ] **Étape 5 : Commit**

```
git add src-tauri/src/catalogue.rs src-tauri/src/providers.rs src-tauri/src/commands.rs
git commit -m "La vue plate du catalogue rend ce que la table rendait"
```

---

### Tâche 4 : La bascule, et la table historique supprimée

**Fichiers :**
- Supprimer : `src-tauri/src/providers.rs`
- Modifier : `src-tauri/src/catalogue.rs`, `lib.rs`, `commands.rs`, `interieur.rs`,
  `package.rs`, `planche.rs`, `projet.rs`, `ebook.rs`, `maquettes.rs`,
  `examples/temoin.rs`, `examples/composer.rs`

`maquettes.rs` et les deux exemples ne figuraient pas dans la première rédaction de ce plan :
ils sont apparus au renommage de la tâche 3. Le compilateur les nommera de toute façon —
c'est écrit ici pour que leur présence au diff ne passe pas pour un débordement.

- [ ] **Étape 1 : Écrire le test qui échoue**

Dans le `mod tests` de `catalogue.rs` :

```rust
/// Le catalogue se sert derrière une référence `'static`, comme la table le faisait :
/// c'est ce qui garde valides les deux signatures de `commands` qui l'exigent.
#[test]
fn un_provider_se_retrouve_par_sa_cle() {
    let pr = provider("bod").expect("bod absent du catalogue");
    assert_eq!(pr.format, (135.0, 215.0));
    assert_eq!(pr.papier_defaut().cle, "creme-90");
    assert!(provider("imprimeur-imaginaire").is_none());
}
```

- [ ] **Étape 2 : Lancer le test pour le voir échouer**

```
cd src-tauri && cargo test --lib catalogue
```

Attendu : ÉCHEC de compilation, `cannot find function provider in this scope`.

- [ ] **Étape 3 : Écrire l'implémentation, puis basculer les appelants**

Dans `catalogue.rs` :

```rust
use std::sync::OnceLock;

/// Le catalogue chargé, une fois pour la vie du processus.
///
/// `OnceLock` et non un état Tauri : deux signatures de `commands` rendent un
/// `&'static Provider`, et une table immuable chargée une fois les satisfait sans que
/// rien d'autre ne change. Hors application — les tests, le témoin —, il s'initialise
/// tout seul sur les seuls fournis.
static PLATS: OnceLock<Vec<Provider>> = OnceLock::new();

/// Tous les couples POD × format connus.
pub fn providers() -> &'static [Provider] {
    PLATS.get_or_init(|| plats().expect("catalogue fourni illisible"))
}

/// Le provider de cette clé, ou `None`.
pub fn provider(cle: &str) -> Option<&'static Provider> {
    providers().iter().find(|p| p.cle == cle)
}
```

Puis remplacer, dans tout le crate, `crate::providers::` par `crate::catalogue::` et
`use crate::providers::{…}` par `use crate::catalogue::{…}` ; `providers::PROVIDERS_HERITEE`
par `catalogue::providers()` dans `commands.rs:156`. Supprimer `src-tauri/src/providers.rs`
et sa ligne dans `lib.rs`. Supprimer le test transitoire
`la_vue_plate_rend_ce_que_la_table_historique_rendait`, qui n'a plus de table à comparer.

**Mais migrer les douze autres tests de son `mod tests`, et non les supprimer avec lui.**
Cette rédaction du plan les avait oubliés, et c'est la faute la plus lourde qu'il ait
portée : ils ne comparent rien à la table, ils **ancrent des valeurs sur des relevés
extérieurs** — le dos de Lulu à 244 pages sur un livre réel tenu en main, le calculateur
BoD à 280 et 560 pages, les gabarits de TheBookEdition à 40, 280 et 750, le calculateur
Bookvault papier par papier, la bascule de gouttière KDP entre 700 et 701 pages, le fond
perdu de chaque gabarit, le refus hors tranche.

Ils n'utilisent que `provider`, `papier_defaut`, `papier`, `gouttiere` et `fond_perdu` —
tous présents à l'identique sur la vue plate. **Contrairement au test de comparaison, ils
ont un après** : une fois la table morte, ce sont eux qui disent que les TOML portent les
bonnes valeurs, le témoin ne composant qu'un livre, chez un seul prestataire, à un seul
format.

Trois d'entre eux recoupent en apparence la validation de la tâche 1. Les garder quand
même : `verifie` contrôle la **forme** de n'importe quel fichier — teinte non vide, bornes
dans l'ordre —, eux contrôlent les **valeurs des six fournis** — colonne de texte au-dessus
de 30 mm, teinte de sept caractères. L'une ne remplace pas l'autre.

Une seule retouche est nécessaire : `assert_eq!(p(f).gouttieres, GOUTTIERES_KDP)` perd sa
constante avec le fichier. Comparer les trois formats KDP entre eux dit la même chose, et
mieux — la garantie cesse de dépendre d'un détail d'écriture de la table.

**Aucune valeur ancrée ni aucun commentaire de documentation ne se réécrit** : leur mérite
est de n'avoir pas été recalculés. Un test qui tomberait après migration ne se corrige pas —
il signalerait que les TOML et les relevés divergent, et c'est une décision, pas un
ajustement.

Les deux signatures de `commands.rs` — `couple` (`:467`) et `papier` (`:1890`) — gardent
`&'static Provider` et `&'static Papier` sans changement : le `OnceLock` les honore.

Ce que le compilateur signalera, et qui est attendu :
- `pr.cle` et `pr.libelle` sont des `String` et non plus des `&'static str`. Les
  `cle: p.cle.into()` de `ProviderVue` **ne compilent pas**, contrairement à ce que cette
  rédaction a d'abord écrit : `String` n'étant pas `Copy`, `into()` déplacerait un champ
  derrière une référence partagée (E0507). Ils passent en `.clone()` — ce qui était de
  toute façon le coût réel, `into()` de `String` vers `String` n'étant qu'un déplacement
  déguisé. Un `pr.cle` passé là où un `&str` est attendu devient `&pr.cle`, et
  `racine.join(pr.cle)` devient `racine.join(&pr.cle)`.
- `pr.gouttieres[0].2` (`interieur.rs:105`) et `pr.papiers` continuent de compiler,
  `Vec<T>` s'indexant et s'itérant comme `&[T]`.
- `PapierVue`, dans `commands.rs`, lit `pa.libelle` : le catalogue dit `nom`. La ligne
  devient `libelle: pa.nom.clone()`. C'est le seul champ renommé de tout le lot.
- `examples/temoin.rs` : `providers::provider(PROVIDER)` devient
  `catalogue::provider(PROVIDER)`, `PROVIDER` restant `"bod"`.

- [ ] **Étape 4 : Lancer toute la vérification**

```
cd src-tauri && cargo test
cd src-tauri && cargo run --example temoin
```

Attendu : tous les tests passent, et le témoin rend **98 pages, dos 7,21 mm**. C'est ici que
la bascule se prouve : le catalogue vient maintenant des TOML, et le livre pagine pareil.

- [ ] **Étape 5 : Commit**

```
git add -A src-tauri/src src-tauri/examples
git commit -m "Le catalogue remplace la table, et le témoin ne bouge pas"
```

---

### Tâche 5 : Les surcharges du poste

**Fichiers :**
- Modifier : `src-tauri/src/catalogue.rs`, `src-tauri/src/lib.rs`

- [ ] **Étape 1 : Écrire les tests qui échouent**

Dans le `mod tests` de `catalogue.rs` :

```rust
use std::io::Write;
use tempfile::TempDir;

/// Écrit un fichier de POD dans le répertoire de surcharges d'un poste d'essai.
fn pose(dir: &TempDir, nom: &str, contenu: &str) {
    let d = dir.path().join("pods");
    std::fs::create_dir_all(&d).unwrap();
    let mut f = std::fs::File::create(d.join(nom)).unwrap();
    f.write_all(contenu.as_bytes()).unwrap();
}

const IMPRIMEUR_ESSAI: &str = r#"
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
"#;

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
    pose(&d, "bod.toml", &IMPRIMEUR_ESSAI.replace(r#"cle = "essai""#, r#"cle = "bod""#));
    let (pods, refus) = charge(Some(d.path()));
    assert!(refus.is_empty(), "{refus:?}");
    assert_eq!(pods.len(), 6, "un remplacement, pas un ajout");
    let bod = pods.iter().find(|p| p.cle == "bod").unwrap();
    assert_eq!(bod.fond_perdu, Some(4.0), "le fourni tient encore");
    assert_eq!(bod.formats.len(), 1);
    assert_eq!(bod.formats[0].mm, (100.0, 150.0));
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
```

- [ ] **Étape 2 : Lancer les tests pour les voir échouer**

```
cd src-tauri && cargo test --lib catalogue
```

Attendu : ÉCHEC de compilation, `cannot find function charge in this scope`.

- [ ] **Étape 3 : Écrire l'implémentation**

Dans `catalogue.rs` :

```rust
use std::path::Path;

/// Un fichier de catalogue que le poste porte et que l'application n'a pas pu lire.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Refus {
    pub fichier: String,
    pub raison: String,
}

/// Là où vivent les surcharges : à côté de `preferences.toml` et de `maquettes/`, parce
/// qu'elles appartiennent à la machine et non au livre.
fn repertoire(config: &Path) -> std::path::PathBuf {
    config.join("pods")
}

/// Les POD du binaire, puis ceux du poste. Même clé : le poste remplace, entièrement.
///
/// Rend aussi ce qui a été refusé, pour que l'interface puisse le dire. Un journal que
/// personne n'ouvre laisserait l'utilisateur devant un catalogue amputé.
pub fn charge(config: Option<&Path>) -> (Vec<Pod>, Vec<Refus>) {
    let mut pods = fournis().expect("catalogue fourni illisible");
    let mut refus = Vec::new();
    let Some(dir) = config.map(repertoire) else {
        return (pods, refus);
    };
    let Ok(entrees) = std::fs::read_dir(&dir) else {
        return (pods, refus);
    };
    let mut chemins: Vec<_> = entrees
        .flatten()
        .map(|e| e.path())
        .filter(|c| c.extension().and_then(|x| x.to_str()) == Some("toml"))
        .collect();
    // Ordre du nom de fichier : deux postes identiques chargent identiquement.
    chemins.sort();
    for chemin in chemins {
        let nom = chemin.display().to_string();
        let lu = std::fs::read_to_string(&chemin)
            .map_err(|e| e.to_string())
            .and_then(|s| Pod::depuis_toml(&s));
        match lu {
            Ok(pod) => match pods.iter().position(|p| p.cle == pod.cle) {
                Some(i) => pods[i] = pod,
                None => pods.push(pod),
            },
            Err(raison) => refus.push(Refus {
                fichier: nom,
                raison,
            }),
        }
    }
    (pods, refus)
}

/// Charge le catalogue une fois, au démarrage de l'application.
///
/// À appeler avant toute commande. Un second appel est un défaut d'ordonnancement et se
/// refuse : sans quoi les fichiers du poste seraient silencieusement ignorés, un
/// `providers()` antérieur ayant déjà figé les seuls fournis.
pub fn initialiser(config: Option<&Path>) -> Result<Vec<Refus>, String> {
    let (pods, refus) = charge(config);
    PLATS
        .set(aplatit(&pods))
        .map_err(|_| "le catalogue a déjà été chargé".to_string())?;
    Ok(refus)
}
```

**`initialiser` n'aura aucun test, et il ne faut surtout pas lui en écrire un ici.** Les
cinq tests ci-dessus passent tous par `charge`, jamais par `initialiser` : c'est ce qui les
rend possibles. `PLATS` est un `OnceLock` de processus, et les quatre cent cinquante tests
de `--lib` partagent un seul processus où des dizaines appellent `provider(…)` — il y est
donc déjà initialisé quand un test d'`initialiser` s'exécuterait, et celui-ci échouerait de
façon non déterministe selon l'ordre d'exécution. Le `PLATS.set`, le refus du second appel
et la ligne de `.setup()` ne sont couverts que par le démarrage réel de l'application. S'il
faut un jour les tester, ce sera dans un binaire d'intégration à part (`src-tauri/tests/`),
qui a son propre processus.

Dans `lib.rs`, en toute première ligne du `.setup(|app| { … })`, avant `menu::poser` :

```rust
use tauri::Manager;
let refus = catalogue::initialiser(app.path().app_config_dir().ok().as_deref())
    .expect("le catalogue doit être chargé avant toute commande");
app.manage(commands::CatalogueRefus(refus));
```

et, dans `commands.rs`, l'état qui les porte :

```rust
/// Les fichiers de catalogue du poste que le démarrage a refusés. Vide sur un poste qui
/// n'en dépose aucun, c'est-à-dire presque toujours.
pub struct CatalogueRefus(pub Vec<crate::catalogue::Refus>);
```

Et, dans `verifie` (`catalogue.rs`), l'exigence que ce test réclame :

```rust
        if !self.reliures.iter().any(|r| r.geometrie.is_some()) {
            return Err(format!(
                "{} : aucune reliure composable. Un POD dont aucune reliure ne porte de \
                 géométrie ne produirait aucun format, et disparaîtrait sans un mot.",
                self.cle
            ));
        }
```

Ce contrôle n'existait pas avant ce lot parce que le cas n'existait pas : la table écrite
en dur ne pouvait pas porter un tel POD. Un fichier déposé, si. **Vérifier que les six
fournis le passent** — ils portent tous une broché.

- [ ] **Étape 4 : Lancer les tests pour les voir passer**

```
cd src-tauri && cargo test --lib catalogue
```

Attendu : tous les tests de `catalogue` passent, les cinq de cette tâche compris.

- [ ] **Étape 5 : Commit**

```
git add src-tauri/src
git commit -m "Le poste peut ajouter un POD, ou en remplacer un"
```

---

### Tâche 6 : Le refus se lit à l'écran

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs`, `src/livraison.js`, `src/app.js`,
  `tests/contrats.test.js`

- [ ] **Étape 1 : Écrire le test qui échoue**

**D'abord, la commande doit exister pour les faux qui démarrent l'application.** Le faux
`invoke` de `contrats.test.js` (`tests/contrats.test.js:44`) lève sur toute commande qu'il ne
connaît pas — c'est sa garde. Neuf fichiers de test démarrent l'application et stubent
`providers_liste` : `composition`, `contrats`, `couverture`, `coquille`, `cycle_de_vie`,
`ebook`, `packages`, `dom_shim`, `epreuve`. Chacun gagne, à côté de sa ligne
`providers_liste` :

```js
  if (cmd === 'catalogue_refus') return [];
```

Sans quoi tous leurs tests lèvent « commande inattendue : catalogue_refus » dès le
démarrage. C'est le couplage que ces gardes existent pour signaler, pas un dommage
collatéral.

Puis, dans `tests/contrats.test.js`, à la suite des contrats existants :

```js
/**
 * Un fichier de catalogue que le démarrage a refusé doit se nommer à l'écran. Un POD
 * absent de la liste sans explication se lirait comme un POD qui n'existe pas — et
 * l'utilisateur chercherait la faute dans son fichier plutôt que dans sa syntaxe.
 */
test('un fichier de catalogue refusé se nomme à la Livraison', async () => {
  const refuse = async (cmd, args) =>
    cmd === 'catalogue_refus'
      ? [{ fichier: '/config/pods/bod.toml', raison: 'expected value at line 2' }]
      : invoke(cmd, args);
  const { els } = await charge({ invoke: refuse, open: async () => null });
  const p = els.get('refusCatalogue');
  assert.equal(p.hidden, false);
  assert.match(p.textContent, /bod\.toml/);
  assert.match(p.textContent, /line 2/);
});

/** Le cas courant : aucun fichier déposé, aucune ligne à l'écran. */
test('sans fichier de catalogue refusé, la ligne reste muette', async () => {
  const { els } = await charge({ invoke, open: async () => null });
  assert.equal(els.get('refusCatalogue').hidden, true);
});
```

- [ ] **Étape 2 : Lancer le test pour le voir échouer**

```
node --test tests/contrats.test.js
```

Attendu : ÉCHEC — `refusCatalogue` est introuvable dans le DOM (`els.get` rend
`undefined`), l'élément n'existant pas encore dans `index.html`.

- [ ] **Étape 3 : Écrire l'implémentation**

Dans `commands.rs` :

```rust
/// Ce que le démarrage a refusé de charger. L'interface le dit à la Livraison : c'est là
/// qu'on regarde la liste des POD, donc là qu'un POD manquant se remarque.
#[tauri::command]
pub fn catalogue_refus(refus: tauri::State<'_, CatalogueRefus>) -> Vec<crate::catalogue::Refus> {
    refus.0.clone()
}
```

et l'inscrire dans le `generate_handler!` de `lib.rs`, à sa place alphabétique.

Dans `src/index.html`, au-dessus de la liste des destinataires de l'étape Livraison :

```html
<p id="refusCatalogue" class="refus" hidden></p>
```

Dans `src/livraison.js` :

```js
/**
 * Les fichiers de catalogue du poste que le démarrage n'a pas pu lire.
 *
 * Muet sur un poste qui n'en dépose aucun — le cas de presque tous. Quand il parle, il
 * nomme le fichier et la raison : un POD absent de la liste sans explication se lirait
 * comme un POD qui n'existe pas.
 */
async function afficherRefusCatalogue() {
  const refus = await invoke('catalogue_refus');
  const p = $('refusCatalogue');
  p.hidden = refus.length === 0;
  p.textContent = refus
    .map((r) => `Catalogue non chargé : ${r.fichier} — ${r.raison}`)
    .join('\n');
}
```

et l'appeler dans `app.js` juste après `providers = await invoke('providers_liste');`
(`src/app.js:454`).

- [ ] **Étape 4 : Lancer le test pour le voir passer**

```
node --test tests/contrats.test.js
```

Attendu : PASS.

- [ ] **Étape 5 : Commit**

```
git add src-tauri/src src src/index.html tests/contrats.test.js
git commit -m "Un fichier de catalogue refusé se nomme à la Livraison"
```

---

### Tâche 7 : Corps, interligne et folio quittent le catalogue

**Fichiers :**
- Modifier : `src-tauri/src/interieur.rs`, `src-tauri/src/catalogue.rs`

- [ ] **Étape 1 : Écrire le test qui échoue**

Dans le `mod tests` de `interieur.rs` :

```rust
    /// Le corps et l'interligne ne sont pas des faits de prestataire : ils étaient
    /// identiques dans les quatorze entrées de la table. Ils vivent ici désormais, et la
    /// source composée les porte quel que soit le gabarit visé — un poche et un grand
    /// format se composent au même corps.
    #[test]
    fn le_corps_et_l_interligne_ne_dependent_pas_du_prestataire() {
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        for cle in ["lulu", "kdp-6x9"] {
            let pr = provider(cle).unwrap();
            let s = source(&livre(), &Interieur::default(), pr, &r, &[], None);
            assert!(s.contains(&format!("size: {CORPS_PT}pt")), "{cle} : {s}");
            assert!(
                s.contains(&format!("leading: {}em", INTERLIGNE - 1.0)),
                "{cle} : {s}"
            );
        }
    }
```

Le montage est celui des tests voisins du fichier — `source(&livre(), &Interieur::default(),
pr, &r, &[], None)`, comme `interieur.rs:771`. `livre()` et `Reglage` sont déjà dans le
`mod tests`.

- [ ] **Étape 2 : Lancer le test pour le voir échouer**

```
cd src-tauri && cargo test --lib interieur
```

Attendu : ÉCHEC de compilation, `cannot find value CORPS_PT in this scope`.

- [ ] **Étape 3 : Écrire l'implémentation**

En tête de `interieur.rs` :

```rust
/// Corps du texte, en points.
///
/// Il vivait dans la table des prestataires, **identique dans ses quatorze entrées** :
/// ce n'est pas un fait d'imprimeur mais un choix typographique. La pagination en dépend,
/// donc le dos : le déplacer est un acte délibéré, à revalider sur un livre réel.
pub const CORPS_PT: f64 = 9.5;

/// Interligne, en multiple du corps. Rapporté à `leading` Typst par `- 1.0`.
pub const INTERLIGNE: f64 = 1.42;

/// Corps du folio, en points.
pub const FOLIO_PT: f64 = 8.0;
```

Remplacer dans `source` : `pr.interligne` → `INTERLIGNE`, `pr.folio_pt` → `FOLIO_PT`,
`pr.corps_pt` → `CORPS_PT`.

Puis retirer les trois champs de `Provider` et de `aplatit` dans `catalogue.rs`, ainsi que
les trois constantes `CORPS_PT`, `INTERLIGNE`, `FOLIO_PT` qui y avaient été posées à la
tâche 3.

- [ ] **Étape 4 : Lancer toute la vérification**

```
cd src-tauri && cargo test
cd src-tauri && cargo run --example temoin
```

Attendu : tous les tests passent, et le témoin rend **98 pages, dos 7,21 mm**. C'est le
contrôle qui compte : le corps et l'interligne ayant déménagé, un dixième perdu en route
changerait la pagination, donc le dos.

- [ ] **Étape 5 : Commit**

```
git add src-tauri/src
git commit -m "Le corps et l'interligne quittent le catalogue pour l'intérieur"
```

---

## À l'œil, avant de clore le lot

Ce que les tests ne prouvent pas, et qui se vérifie dans l'application lancée
(`cd src-tauri && cargo tauri dev`) :

1. L'étape Livraison affiche **la même liste de quatorze prestataires** qu'avant le lot, aux
   mêmes libellés, dans le même ordre.
2. Générer un package sur un livre réel donne les mêmes fichiers, aux mêmes noms.
3. Déposer `<config>/pods/mon-imprimeur.toml` — le `IMPRIMEUR_ESSAI` du test fait l'affaire —
   puis relancer : le POD paraît dans la liste, **sans recompilation**. Sous macOS, le
   répertoire est `~/Library/Application Support/<identifiant de l'application>/pods/`.
4. Y introduire une faute de frappe, relancer : l'application démarre, la liste garde ses
   quatorze entrées, et la ligne de refus nomme le fichier.

## Ce que ce lot ne fait pas

Le livrable à cinq axes, la migration du `.ozalid`, la cascade à l'écran et le remplissage
du catalogue de BoD sont les lots 2 à 4 de la spec. Ici, `cle_heritee` tient encore
l'identité plate, aucune reliure hors broché n'est écrite, et l'écran ne change pas.
