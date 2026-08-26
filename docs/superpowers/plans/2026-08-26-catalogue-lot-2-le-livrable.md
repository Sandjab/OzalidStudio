# Catalogue lot 2 — le livrable

> **Pour un exécutant agentique :** SOUS-COMPÉTENCE REQUISE : `superpowers:subagent-driven-development`
> (recommandé) ou `superpowers:executing-plans` pour exécuter ce plan tâche par tâche. Les
> étapes sont des cases à cocher (`- [ ]`).

**But :** un destinataire devient un **livrable** — l'identité à quatre axes (POD, format,
reliure, papier), le `.ozalid` migré en v5, la mesure rangée sous le gabarit d'intérieur
(POD, format, reliure), les noms de packages dérivés du livrable, les commandes
`livrable_*`. Deux livrables du même POD coexistent dès qu'un axe diffère, et comparer
deux papiers ne coûte plus une composition.

**Architecture :** `Provider` **reste le type de calcul** (verdict 1c de la reconnaissance :
le `&'static` tombe pour neuf esperluettes et deux `clone()`, et `interieur`, `planche`,
`package`, `ebook` ne bougent pas). Le catalogue expose `resout(&Fabrication) -> Resolu`
(quatre références `&'static` dans les `Pod` chargés) et `Resolu::provider()` fabrique la
vue plate à la volée. La mesure vit dans `Livraison.mesures`, une `BTreeMap` à clé de
gabarit `pod-format-reliure`, et **perd son champ `dos`** : le dos se recalcule par
livrable depuis la formule de son papier — c'est ce qui permet à deux papiers de partager
la même mesure. Une `LivraisonVue` côté Rust recalcule `compose` sur chaque livrable : le
front ne voit du déplacement de la mesure que le renommage de l'identité. La bascule se
fait en trois temps compilables : la donnée d'abord (écran inchangé, tâche 4), la clé de la
vue plate ensuite (tâche 5), les commandes et le front enfin (tâche 6).

**Pile :** Rust 2021, Tauri 2, `serde` + `toml 0.8`, `tempfile`. Front vanilla. Tests :
`cargo test` depuis `src-tauri/`, `node --test tests/*.test.js` depuis la racine,
`cargo run --example temoin` comme témoin.

**Spec :** `docs/superpowers/specs/2026-08-26-catalogue-et-livrables-design.md` (§ 4 à 7).
**Reconnaissance :** `docs/superpowers/2026-08-26-reconnaissance-lot-2.md` — les verdicts
cités dans ce plan (1a–5g, § 8) y sont, chacun vérifié au compilateur.

---

## Décisions arbitrées (utilisateur, 26/08) — ne pas les rouvrir

1. **Le répertoire de package garde les clés telles quelles** :
   `bod-135x215-broche-creme-90`, cinq segments visibles, aucune transformation. La spec
   écrivait `creme90` ; c'est elle qui sera corrigée (tâche 8). Une clé se fabrique et se
   compare, elle ne se découpe **jamais** — le séparateur vit déjà dans les valeurs
   (`creme-90`, `tbe-110x170`).
2. **Une `LivraisonVue` recalcule `compose` par livrable.** Le front lit la même forme
   qu'aujourd'hui (`compose.pages`, `compose.dos`…) ; seul le nom de l'identité change.
3. **Le test d'ancrage des quatorze libellés est converti**, pas retiré : il fige les
   quatorze livrables par défaut (clé de gabarit, libellé, papier par défaut) jusqu'à ce
   que le lot 3 remplace la liste par la cascade.

Et les verdicts de la reconnaissance, repris tels quels : `courant` est une **clé**, jamais
un index (verdict 2 — un index décalé fait boucler la composition sans erreur) ; pas de
`#[serde(untagged)]` pour la migration (verdict 3 — il perd le nom du champ fautif) ;
`est_un_nom` s'étend aux clés de POD, format, reliure, finition et papier (verdict 4 — ce
sont elles qui nomment les répertoires quand `cle_heritee` disparaît) ; la politique
d'invalidation des mesures se traite ici (spec § 8) : la mesure porte l'**empreinte** de
son gabarit, et un gabarit réécrit sur le poste périme la mesure à l'ouverture.

## Avant chaque commit

Valables pour **toutes** les étapes « Commit », sans être répétées. Depuis `src-tauri/` :

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Depuis la racine : `node --test tests/*.test.js`.

Et `cargo test -- --ignored` (depuis `src-tauri/`, ~1,3 s) si `package.rs`,
`interieur.rs`, `planche.rs` ou `typst.rs` a changé : quatre tests y composent
réellement avec le sidecar, dont le **seul** qui prouve qu'un gabarit ne compose son
intérieur qu'une fois — `cargo test` nu reste vert quand la mémoïsation disparaît
(mesuré à la tâche 3).

Et, tout fichier de `src-tauri/` ayant changé dans la tâche :
`cd src-tauri && cargo run --example temoin`. Attendu, à chaque tâche, sans exception :
**98 pages, dos 7,21 mm**. Un écart n'est pas à corriger dans le témoin : c'est le signe
que la conversion a perdu une valeur.

## Pièges transverses, relevés par la reconnaissance

- **Tout littéral Rust contenant un TOML de POD avec une `teinte` doit être `r##"…"##`** :
  `teinte = "#f7f0e0"` contient `"#`, qui ferme un `r#"…"#` (verdict 5a, reproduit).
- **`Mesure` n'est pas `Copy`** : toute lecture derrière une référence demande `.clone()`
  (verdict 5f).
- **Un renommage JS incohérent ne fait pas échouer la suite, il la fait boucler** sans fin
  (verdict § 6) : ne jamais changer le faux Rust d'un fichier de test sans changer les
  lecteurs du même coup, et réciproquement. Si `node --test` ne rend pas la main en une
  minute, c'est ce bug-là.
- **`src-tauri/tests/catalogue_initialise.rs` n'apparaît dans aucun grep sur `src/`**
  (verdict 5g) : il bouge aux tâches 5 et 7, et il ne porte qu'un seul `#[test]` —
  `initialiser` pose un `OnceLock` par processus.
- **La clé du POD TheBookEdition est `tbe`**, pas `thebookedition` (le fichier, lui,
  s'appelle `thebookedition.toml`).
- Après un changement de `src/` seul : `touch src-tauri/src/lib.rs` avant `cargo build`.

## Structure des fichiers

| Fichier | Responsabilité dans ce lot |
|---|---|
| `src-tauri/src/catalogue.rs` | `Fabrication`, `Resolu`, `resout`, `Resolu::provider()`, `Resolu::empreinte()`, `PODS`/`pods()`/`pod()`, `HERITEES`, `est_un_nom` généralisé, `aplatit` re-clé, `provider()` relégué aux tests |
| `src-tauri/src/projet.rs` | `Livrable`, `Livraison` (livrables + `mesures`), `Mesure` sans `dos` avec empreinte, `normalise`, migration v4→v5 |
| `src-tauri/src/commands.rs` | `vise` sans `&'static`, commandes `livrable_*`, `LivraisonVue`/`LivrableVue`/`MesureVue`, `ProviderVue` re-clé, `packager` par lot |
| `src-tauri/src/package.rs` | `nom(cle, …)`, `InterieurCompose`, `composer_interieur`, `assembler` à intérieur injecté, `lot` mémoïsé, `Cible` |
| `src-tauri/src/lib.rs` | le `generate_handler` renommé |
| `src-tauri/pods/*.toml` | les 14 lignes `cle_heritee` supprimées (tâche 7) |
| `src-tauri/tests/catalogue_initialise.rs` | suit `PODS` et les nouvelles clés |
| `src-tauri/examples/{temoin,composer,packager,ebook}.rs` | le triplet remplace la clé plate |
| `src/{app,livraison,couverture,envois}.js` | `livrables`/`cle` remplacent `destinataires`/`provider` ; garde de veille durcie |
| `tests/*.test.js` | faux Rust et fixtures au nouveau format (9 fichiers, `placement.test.js` épargné) |
| `docs/superpowers/specs/2026-08-26-catalogue-et-livrables-design.md` | trois corrections de fait (tâche 8) |

---

### Tâche 1 : L'identité à quatre axes (`catalogue.rs`)

**Fichiers :**
- Modifier : `src-tauri/src/catalogue.rs` (imports serde, puis ajouts en fin de partie code, avant `mod tests`)
- Modifier : `src-tauri/pods/bod.toml` — le bloc `[[reliure]]` rigide non outillé, que la
  spec § 2 écrit en exemple et que le lot 1 n'a jamais porté dans le fichier réel :

```toml
[[reliure]]
cle = "rigide"
nom = "Couverture rigide"
non_outille = "géométrie du casewrap non relevée : rempli, mors, épaisseur des cartons"
```

  (après le bloc `broche` ; sans effet sur `aplatit` ni `resout` — `broche` reste la
  première et seule reliure composable, et c'est ce que le test de refus exerce)
- Tests : le `mod tests` du même fichier

Rien de ce que cette tâche ajoute n'est encore consommé hors tests : elle pose l'identité,
sa résolution et la fabrique de `Provider`, que les tâches 3 à 6 branchent.

- [ ] **Étape 1 : écrire les tests, les voir rouges**

Dans le `mod tests` de `catalogue.rs` (les types n'existant pas, le rouge est une erreur de
compilation — c'est le rouge attendu) :

```rust
/* ---------- l'identité à quatre axes ---------- */

fn fabrication(pod: &str, format: &str, reliure: &str, papier: &str) -> Fabrication {
    Fabrication {
        pod: pod.into(),
        format: format.into(),
        reliure: reliure.into(),
        papier: papier.into(),
    }
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
    assert!(e.contains("rigide") || e.contains("Couverture rigide"), "{e}");
    assert!(e.contains("non relevée"), "la raison du fichier doit traverser : {e}");
}

/// L'ancrage de la fabrique : pour chacune des quatorze clés héritées, le `Provider`
/// fabriqué depuis le triplet (et le papier par défaut du POD) est **identique champ
/// par champ** à celui de la vue plate — la clé exceptée, qui change de convention à la
/// tâche 5. C'est ce test qui prouve que remplacer `aplatit` par `resout` + `provider()`
/// ne déplace aucune valeur.
#[test]
fn le_livrable_resolu_fabrique_le_provider_de_la_vue_plate() {
    for (heritee, pod, format, reliure) in HERITEES {
        let plat = provider(heritee).unwrap_or_else(|| panic!("clé plate absente : {heritee}"));
        let papier = &pod_de(pod).papiers[0].cle;
        let fait = resout(&fabrication(pod, format, reliure, papier)).unwrap().provider();
        assert_eq!(fait.libelle, plat.libelle);
        assert_eq!(fait.format, plat.format);
        assert_eq!(fait.marge_haut, plat.marge_haut);
        assert_eq!(fait.marge_bas, plat.marge_bas);
        assert_eq!(fait.exterieur, plat.exterieur);
        assert_eq!(fait.gouttieres, plat.gouttieres);
        assert_eq!(fait.fond_perdu, plat.fond_perdu);
        assert_eq!(fait.pages_min, plat.pages_min);
        assert_eq!(fait.pages_max, plat.pages_max);
        assert_eq!(fait.papiers, plat.papiers);
    }
}

fn pod_de(cle: &str) -> &'static Pod {
    pod(cle).unwrap_or_else(|| panic!("POD inconnu : {cle}"))
}

/// La table de migration est ancrée sur les fichiers : chaque ligne désigne un triplet
/// qui se résout, et la clé héritée qu'elle remplace est bien celle que le format porte
/// encore. La seconde moitié tombe à la tâche 7 avec `cle_heritee`.
#[test]
fn chaque_cle_heritee_a_son_triplet() {
    assert_eq!(HERITEES.len(), 14);
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
```

> Les marges exactes de l'empreinte BoD sont celles de `src-tauri/pods/bod.toml`
> (`haut = 18.8`, `bas = 28.0`, `exterieur = 15.0`, gouttière unique `[[24, 900, 20.0]]`) :
> si le fichier en dit d'autres, c'est le fichier qui fait foi — relever la valeur, pas la
> forcer.

- [ ] **Étape 2 : lancer, constater le rouge**

`cd src-tauri && cargo test la_cle_d_un_livrable -- --nocapture` — attendu : échec de
compilation (`cannot find type Fabrication`).

- [ ] **Étape 3 : implémenter**

Dans `catalogue.rs`. D'abord l'import (`Serialize` manque) :

```rust
use serde::{Deserialize, Serialize};
```

Puis, après la section `Provider` existante :

```rust
/* ---------- l'identité d'un livrable ---------- */

/// L'identité de fabrication d'un livrable : les quatre axes qui changent le fichier
/// produit. La finition n'y est pas — mat ou brillant donnent le même PDF (spec § 4).
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
        format!("{}-{}-{}-{}", self.pod, self.format, self.reliure, self.papier)
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
/// test `provider`. Elle ne grandit plus — les clés neuves naissent en triplet.
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
/// profondeur : c'est lui que `resout` interroge. `initialiser` pose les deux depuis le
/// même chargement ; hors application, les seuls fournis.
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
    let pod = pod(&f.pod).ok_or_else(|| format!("POD inconnu : {}", f.pod))?;
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
            Some(raison) => format!("{} : {raison}", reliure.nom),
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
            cle: Fabrication {
                pod: self.pod.cle.clone(),
                format: self.format.cle.clone(),
                reliure: self.reliure.cle.clone(),
                papier: self.papier.cle.clone(),
            }
            .cle_gabarit(),
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
```

Et `initialiser` pose les deux tables depuis le même chargement :

```rust
pub fn initialiser(config: Option<&Path>) -> Result<Vec<Refus>, String> {
    let (pods, refus) = charge(config);
    PLATS
        .set(aplatit(&pods))
        .map_err(|_| "le catalogue a déjà été chargé".to_string())?;
    PODS.set(pods)
        .map_err(|_| "le catalogue a déjà été chargé".to_string())?;
    Ok(refus)
}
```

- [ ] **Étape 4 : vérifier le vert**

`cargo test --lib catalogue` — les nouveaux tests passent, les 464 existants aussi.

- [ ] **Étape 5 : Commit**

`git add src-tauri/src/catalogue.rs` puis commit :
« L'identité d'un livrable tient en quatre clés, et se résout contre le catalogue »

---

### Tâche 2 : Toutes les clés deviennent des noms

**Fichiers :**
- Modifier : `src-tauri/src/catalogue.rs` (`verifie`, `cle_non_vide` supprimée)

Le trou du verdict 4, mesuré à la sonde : `pod.cle = "../evade"`,
`papier.cle = "../../ailleurs"`, `format.cle = "C:nul*"` sont **acceptés** aujourd'hui,
parce que `est_un_nom` ne s'applique qu'à `cle_heritee`. Or ce sont ces clés-là qui
nomment les répertoires de package à partir de la tâche 4. Refuser à la lecture vaut mieux
que slugifier à l'écriture : le fichier fautif se nomme à la Livraison, et l'utilisateur
voit quelle clé corriger.

- [ ] **Étape 1 : écrire les tests, les voir rouges**

Dans le `mod tests` de `catalogue.rs`, à côté des refus existants (reprendre le socle
`FORMAT`/`RELIURE`/`PAPIER` des constantes de test déjà en place ; **attention au
`r##"…"##`** si le socle porte une `teinte`) :

```rust
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
gouttieres = [[1, 900, 10.0]]
{RELIURE}{PAPIER}"##
    ))
    .unwrap_err();
    assert!(e.contains("C:nul*"), "{e}");
}
```

> Les socles `FORMAT`, `RELIURE`, `PAPIER` existent déjà en tête du `mod tests` — les
> réutiliser tels quels ; n'écrire en clair que le bloc mis en cause, comme les tests de
> refus en place. Adapter les trois littéraux ci-dessus à leur contenu exact.

Lancer : rouge (les trois passent la lecture aujourd'hui).

- [ ] **Étape 2 : implémenter**

Dans `verifie`, remplacer la boucle `cle_non_vide` et le contrôle du POD par `est_un_nom`,
et supprimer `cle_non_vide` (le vide est un non-nom comme un autre) :

```rust
if !est_un_nom(&self.cle) {
    return Err(format!(
        "clé de POD « {} » : minuscules, chiffres et tirets, rien d'autre — elle nomme \
         des répertoires et des identifiants.",
        self.cle
    ));
}
for (quoi, cle) in self
    .formats
    .iter()
    .map(|f| ("un format", f.cle.as_str()))
    .chain(self.reliures.iter().map(|r| ("une reliure", r.cle.as_str())))
    .chain(self.finitions.iter().map(|f| ("une finition", f.cle.as_str())))
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
```

Le doc-commentaire d'`est_un_nom` change de sujet : il ne parle plus de la seule clé
héritée mais de **toute** clé du catalogue.

- [ ] **Étape 3 : réparer les tests de refus existants**

Les tests qui attendaient « sans clé » (clé vide) doivent maintenant attendre le nouveau
message. Chercher : `grep -n "sans clé" src-tauri/src/catalogue.rs` — adapter chaque
assertion au message réel, sans affaiblir ce qu'elle vérifie (le refus doit toujours
**nommer la clé et le bloc**). Vérifier aussi qu'aucun fichier fourni ne tombe :
`cargo test --lib catalogue` doit rendre `les_six_fichiers_fournis_se_lisent` vert
(toutes les clés des six fichiers sont déjà `[a-z0-9-]`, `tbe` papier `120` compris).

- [ ] **Étape 4 : Commit**

« Une clé de catalogue est un nom, quel que soit l'axe qui la porte »

---

### Tâche 3 : Le nom des sorties vient d'une clé, l'intérieur se compose une fois par gabarit (`package.rs`)

**Fichiers :**
- Modifier : `src-tauri/src/package.rs` (`nom`, `assembler`, `assembler_envois`, + `InterieurCompose`, `composer_interieur`, `Cible`, `lot`)
- Modifier : `src-tauri/src/commands.rs` (`packager` passe par `lot` ; le site des envois passe la clé)
- Modifier : `src-tauri/examples/temoin.rs`, `src-tauri/examples/packager.rs` (nouvelle signature)
- Tests : `mod tests` de `package.rs`

Verdict 5c : il n'existe **aucun** mécanisme de mémoïsation à étendre — `assembler`
appelle `interieur::converge` sans condition. Cette tâche sépare l'intérieur (par
gabarit) de la planche (par livrable), et donne au packager un lot mémoïsé. Les appelants
passent la clé du prestataire actuel (`pr.cle`) : le comportement est identique tant que
l'identité n'a pas basculé, et c'est ce que le témoin prouve.

- [ ] **Étape 1 : écrire les tests, les voir rouges**

```rust
/// Le rempla­cement du test `les_sorties_portent_la_cle_du_prestataire` : le nom vient
/// désormais d'une clé de livrable entière, cinq segments compris — deux packages
/// ouverts côte à côte ne peuvent pas être confondus, deux papiers non plus.
#[test]
fn les_sorties_portent_la_cle_du_livrable() {
    assert_eq!(
        nom("bod-135x215-broche-creme-90", "couverture", "pdf"),
        "couverture-bod-135x215-broche-creme-90.pdf"
    );
    assert_eq!(
        nom("bod-135x215-broche-creme-90", "interieur", "typ"),
        "interieur-bod-135x215-broche-creme-90.typ"
    );
}

/// Un intérieur déjà composé n'est pas recomposé : `assembler` reçoit l'intérieur d'un
/// gabarit et ne rappelle Typst que pour la planche. Preuve par l'ordre des refus : avec
/// un binaire Typst inexistant **et** un intérieur prêt, l'échec doit venir de la
/// maquette absente (étape planche) — si l'intérieur était recomposé, il viendrait de
/// Typst, avant elle.
#[test]
fn un_interieur_pret_n_est_pas_recompose() {
    let projet = Projet::nouveau(livre_d_essai(), "# Un\n\nParagraphe.".into());
    // Pas de maquette de couverture : c'est le refus attendu.
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("interieur-essai.typ");
    let pdf = dir.path().join("interieur-essai.pdf");
    std::fs::write(&src, "source factice").unwrap();
    std::fs::write(&pdf, "pdf factice").unwrap();
    let pret = InterieurCompose {
        pages: 100,
        gouttiere: 20.0,
        blanche: false,
        polices_introuvables: vec![],
        src,
        pdf,
    };
    let pr = provider_d_essai();
    let e = assembler(
        &projet,
        &pr,
        &pr.papiers[0],
        Releve::default(),
        "essai",
        &pret,
        dir.path(),
        &Typst::new("typst-qui-n-existe-pas"),
    )
    .unwrap_err();
    assert!(e.contains("maquette"), "l'intérieur a été recomposé : {e}");
}

/// Spec § 9 : deux livrables du même gabarit d'intérieur ne déclenchent **qu'une**
/// composition. Composition réelle (Typst du PATH, comme `interieur.rs` le fait déjà) :
/// le second package porte un intérieur copié, pas recomposé, et les deux PDF sont
/// identiques à l'octet.
#[test]
fn deux_livrables_du_meme_gabarit_ne_composent_l_interieur_qu_une_fois() {
    let mut projet = Projet::nouveau(livre_d_essai(), "# Un\n\nParagraphe.".into());
    projet.meta.couverture.maquette = Some(
        crate::maquettes::par_cle(None, "filets")
            .expect("maquette fournie « filets »")
            .couverture,
    );
    let racine = tempfile::tempdir().unwrap();
    let typst = Typst::new("typst")
        .avec_polices(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts"));
    let pr = provider_d_essai();
    let creme = pr.papiers[0].clone();
    let blanc = Papier {
        cle: "blanc-essai".into(),
        nom: "Blanc d'essai".into(),
        teinte: "#ffffff".into(),
        dos: crate::catalogue::Dos::Multiplie { par: 0.08, plus: 0.0 },
        source: None,
    };
    let cibles = [
        Cible {
            pr: pr.clone(),
            papier: creme,
            releve: Releve::default(),
            cle: "essai-livre-broche-creme".into(),
            cle_gabarit: "essai-livre-broche".into(),
        },
        Cible {
            pr,
            papier: blanc,
            releve: Releve::default(),
            cle: "essai-livre-broche-blanc-essai".into(),
            cle_gabarit: "essai-livre-broche".into(),
        },
    ];
    let sorties = lot(&projet, &cibles, racine.path(), &typst);
    let [a, b]: [&Package; 2] = [
        sorties[0].as_ref().expect("premier package"),
        sorties[1].as_ref().expect("second package"),
    ];
    assert!(!a.interieur_partage, "le premier compose");
    assert!(b.interieur_partage, "le second copie");
    assert_eq!(a.pages, b.pages);
    assert!(b.dos > a.dos, "le papier plus épais fait un dos plus épais");
    let lu = |cle: &str| {
        std::fs::read(racine.path().join(cle).join(format!("interieur-{cle}.pdf"))).unwrap()
    };
    assert_eq!(
        lu("essai-livre-broche-creme"),
        lu("essai-livre-broche-blanc-essai"),
        "le même intérieur, à l'octet"
    );
}
```

Avec deux aides de test, un `Provider` **synthétique** (aucune dépendance au catalogue :
les bornes des vrais POD refusent un manuscrit d'une page) et un `Livre` minimal :

```rust
fn provider_d_essai() -> Provider {
    Provider {
        cle: "essai-livre-broche".into(),
        libelle: "Essai — livre".into(),
        format: (135.0, 215.0),
        marge_haut: 18.8,
        marge_bas: 28.0,
        exterieur: 15.0,
        gouttieres: vec![(1, 900, 20.0)],
        fond_perdu: Some(5.0),
        pages_min: 1,
        pages_max: 900,
        papiers: vec![Papier {
            cle: "creme".into(),
            nom: "Crème d'essai".into(),
            teinte: "#f7f0e0".into(),
            dos: crate::catalogue::Dos::Multiplie { par: 0.0675, plus: 0.6 },
            source: None,
        }],
    }
}

fn livre_d_essai() -> crate::projet::Livre {
    // Le `Livre` du témoin, réduit : chercher d'abord un constructeur réutilisable dans
    // le `mod tests` (celui de `projet_en_images`) et le préférer s'il existe.
    crate::projet::Livre {
        titre: "Essai".into(),
        titre_page: "%TITRE%".into(),
        auteur: "Autrice".into(),
        genre: "essai".into(),
        editeur: "Editeur".into(),
        collection: "Collection".into(),
        monogramme: "M".into(),
        copyright: "Domaine public.".into(),
        prix: "Prix".into(),
        mention: "Mention".into(),
        dedicace: String::new(),
        chapitres: Some(1),
    }
}
```

> `livre_d_essai` : chercher d'abord un constructeur existant dans le `mod tests` de
> `package.rs` (`projet_en_images` en fabrique un — en extraire le `Livre`). Ne créer le
> nôtre que s'il n'y en a pas de réutilisable. Et **`r##`** sur tout littéral qui
> porterait une teinte.

Lancer : rouge (signatures inexistantes).

- [ ] **Étape 2 : implémenter dans `package.rs`**

```rust
/// Nom de fichier des sorties d'un livrable. Le nom porte la clé entière : deux
/// packages ouverts côte à côte ne peuvent pas être confondus, deux papiers non plus.
fn nom(cle: &str, quoi: &str, ext: &str) -> String {
    format!("{quoi}-{cle}.{ext}")
}

/// L'intérieur composé d'un gabarit : ce que deux livrables du même gabarit partagent.
/// La planche, elle, reste par livrable — le dos suit le papier.
#[derive(Debug, Clone)]
pub struct InterieurCompose {
    pub pages: u32,
    pub gouttiere: f64,
    pub blanche: bool,
    pub polices_introuvables: Vec<String>,
    pub src: PathBuf,
    pub pdf: PathBuf,
}

/// Compose l'intérieur : la convergence, puis le PDF. C'est le bloc 1 de l'ancien
/// `assembler`, sorti pour n'être payé qu'une fois par gabarit.
pub fn composer_interieur(
    projet: &Projet,
    pr: &Provider,
    cle: &str,
    dossier: &Path,
    typst: &Typst,
) -> Result<InterieurCompose, String> {
    let int = &projet.meta.interieur;
    int.verifie()?;
    std::fs::create_dir_all(dossier)
        .map_err(|e| format!("répertoire inutilisable ({}) : {e}", dossier.display()))?;
    let livre = &projet.meta.livre;
    let chapitres = manuscrit::decoupe(&projet.texte, livre.chapitres)?;

    let src = dossier.join(nom(cle, "interieur", "typ"));
    let r = interieur::converge(pr, |reglage| {
        ecrire(&src, &interieur::source(livre, int, pr, reglage, &chapitres, None))?;
        typst.pages(&src)
    })?;
    let reglage = Reglage {
        gouttiere: r.gouttiere,
        blanche: r.blanche,
    };
    ecrire(&src, &interieur::source(livre, int, pr, &reglage, &chapitres, None))?;
    let pdf = dossier.join(nom(cle, "interieur", "pdf"));
    let polices_introuvables = typst.compile(&src, &pdf)?;
    Ok(InterieurCompose {
        pages: r.pages,
        gouttiere: r.gouttiere,
        blanche: r.blanche,
        polices_introuvables,
        src,
        pdf,
    })
}
```

`assembler` change de signature — il reçoit la clé et l'intérieur, et n'appelle plus la
convergence :

```rust
pub fn assembler(
    projet: &Projet,
    pr: &Provider,
    papier: &Papier,
    releve: Releve,
    cle: &str,
    interieur: &InterieurCompose,
    dossier: &Path,
    typst: &Typst,
) -> Result<Package, String> {
    let livre = &projet.meta.livre;
    std::fs::create_dir_all(dossier)
        .map_err(|e| format!("répertoire inutilisable ({}) : {e}", dossier.display()))?;

    // 1. L'intérieur du gabarit, composé ailleurs ou ici : s'il vient d'un autre
    // répertoire, il est copié sous le nom de ce livrable — les octets sont les mêmes,
    // et c'est le sens de « une seule composition ».
    let src_int = dossier.join(nom(cle, "interieur", "typ"));
    let pdf_int = dossier.join(nom(cle, "interieur", "pdf"));
    let interieur_partage = interieur.pdf != pdf_int;
    if interieur_partage {
        for (de, vers) in [(&interieur.src, &src_int), (&interieur.pdf, &pdf_int)] {
            std::fs::copy(de, vers)
                .map_err(|e| format!("copie de l'intérieur ({}) : {e}", vers.display()))?;
        }
    }
    if interieur.pages < pr.pages_min || interieur.pages > pr.pages_max {
        return Err(format!(
            "{cle} : {} pages, hors des {} à {} que {} accepte en dos carré collé.",
            interieur.pages, pr.pages_min, pr.pages_max, pr.libelle
        ));
    }
    // NB : à la tâche 5, quand `Provider` porte sa `fabrication`, ce message cite la
    // reliure (`en {}`, `pr.fabrication.reliure`) au lieu du « dos carré collé »
    // générique — c'est la spec § 7, et la tâche 5 le rappelle.
    let mut polices_introuvables = interieur.polices_introuvables.clone();

    // 2. Le dos découle de cette pagination-là, jamais d'une saisie.
    let g = Gabarit::pour(pr, papier, interieur.pages, releve)?;

    // 3. La planche. (Bloc inchangé, aux noms près : `nom(cle, …)`.)
    ...
}
```

Le bloc 3 (planche + vignette) est repris tel quel de l'`assembler` actuel, en remplaçant
chaque `nom(pr, …)` par `nom(cle, …)` et `r.pages`/`r.gouttiere`/`r.blanche` par les
champs d'`interieur`. Le `Package` rendu change deux choses :

```rust
    Ok(Package {
        cle: cle.to_string(),          // était `provider: pr.cle.clone()`
        ...
        interieur_partage,             // champ neuf : cet intérieur est une copie
        ...
    })
```

`Package` : renommer le champ `provider` en `cle` (le front ne le lit nulle part — vérifié
par grep, seules les fixtures de test le construisent) et ajouter
`pub interieur_partage: bool` avec son doc-commentaire (« Vrai quand l'intérieur de ce
package est la copie de celui d'un autre livrable du même gabarit : il n'a pas été
recomposé. »).

Puis le lot :

```rust
/// Un livrable prêt à packager : sa vue plate, son papier, son relevé et ses clés.
pub struct Cible {
    pub pr: Provider,
    pub papier: Papier,
    pub releve: Releve,
    pub cle: String,
    pub cle_gabarit: String,
}

/// Packager un lot de livrables, l'intérieur composé **une fois par gabarit**.
///
/// Le premier livrable d'un gabarit compose dans son répertoire ; les suivants copient.
/// Un échec de composition ne condamne pas le gabarit : le suivant du même gabarit
/// réessaie, faute d'entrée retenue.
pub fn lot(
    projet: &Projet,
    cibles: &[Cible],
    racine: &Path,
    typst: &Typst,
) -> Vec<Result<Package, String>> {
    let mut prets: std::collections::BTreeMap<String, InterieurCompose> =
        std::collections::BTreeMap::new();
    cibles
        .iter()
        .map(|c| {
            let dossier = racine.join(&c.cle);
            let interieur = match prets.get(&c.cle_gabarit) {
                Some(i) => i.clone(),
                None => {
                    let i = composer_interieur(projet, &c.pr, &c.cle, &dossier, typst)?;
                    prets.insert(c.cle_gabarit.clone(), i.clone());
                    i
                }
            };
            assembler(
                projet, &c.pr, &c.papier, c.releve, &c.cle, &interieur, &dossier, typst,
            )
        })
        .collect()
}
```

`assembler_envois` : son package de référence passe par les deux temps —

```rust
    let reference = racine.join(".reference");
    let int = composer_interieur(projet, pr, cle, &reference, typst)?;
    let base = assembler(projet, pr, papier, releve, cle, &int, &reference, typst)?;
```

— et la fonction gagne le paramètre `cle: &str` qu'elle transmet (sa boucle par envoi
compose des intérieurs **avec trace**, distincts par nature : elle garde son propre
chemin, seuls les `nom(pr, …)` deviennent `nom(cle, …)`).

- [ ] **Étape 3 : adapter les appelants, à comportement constant**

- `commands::packager` : la boucle actuelle construit des `Cible` puis appelle `lot` —
  les erreurs de résolution (`provider` inconnu, papier inconnu) restent des `Resultat`
  d'erreur fabriqués avant le lot :

```rust
    let mut cibles = Vec::new();
    let mut prealables = Vec::new(); // (index, Resultat d'erreur)
    for (i, d) in destinataires.iter().enumerate() {
        match catalogue::provider(&d.provider)
            .ok_or_else(|| format!("prestataire inconnu : {}", d.provider))
            .and_then(|pr| Ok((pr, papier(pr, Some(&d.papier))?)))
        {
            Ok((pr, pa)) => cibles.push((
                i,
                package::Cible {
                    pr: pr.clone(),
                    papier: pa.clone(),
                    releve: planche::Releve { dos: d.dos_mm, fond_perdu: d.fond_perdu_mm },
                    cle: pr.cle.clone(),         // compat : la clé plate, jusqu'à la tâche 4
                    cle_gabarit: pr.cle.clone(), // idem — un gabarit par destinataire
                },
            )),
            Err(e) => prealables.push((i, e)),
        }
    }
    let racine = sorties_racine(o)?;
    let paquets = package::lot(&o.projet, &cibles.iter().map(|(_, c)| c.clone()).collect::<Vec<_>>(), &racine, &typst);
```

  (ou toute écriture plus directe qui préserve l'ordre des `Resultat` — l'important : les
  libellés et erreurs par destinataire ne changent pas, et `sorties_dossier` n'est plus
  appelé par `packager`, `lot` fabriquant `racine.join(cle)` lui-même). `Cible` doit
  dériver `Clone` si cette écriture l'exige.
- Le site des envois (`commands.rs`, autour de 1875) passe `&pr.cle` au nouveau paramètre
  `cle` d'`assembler_envois`.
- `examples/temoin.rs` et `examples/packager.rs` : deux temps —

```rust
    let int = package::composer_interieur(&projet, pr, &pr.cle, &sortie, &typst)?;
    let p = package::assembler(
        &projet, pr, pr.papier_defaut(), Releve::default(), &pr.cle, &int, &sortie, &typst,
    )?;
```

- Le test `les_sorties_portent_la_cle_du_prestataire` (`package.rs:497`) est **supprimé**,
  remplacé à l'étape 1 — c'est un retrait délibéré, son successeur est nommé.

- [ ] **Étape 4 : vérifier**

`cargo test --lib package` vert, puis la suite entière, puis le témoin : **98 pages,
dos 7,21 mm** — l'intérieur composé en deux temps ne déplace rien.

- [ ] **Étape 5 : Commit**

« Le nom d'une sortie vient d'une clé de livrable, et l'intérieur se compose une fois par gabarit »

---

### Tâche 4 : Le projet porte des livrables, la mesure vit sous le gabarit, le `.ozalid` migre en v5

**Fichiers :**
- Modifier : `src-tauri/src/projet.rs` (`Destinataire` → `Livrable`, `Livraison`, `Mesure`, `normalise`, `migre`, `VERSION`)
- Modifier : `src-tauri/src/commands.rs` (`vise`, les quatre commandes en **compat**, `composer`, `packager`, `vue`, `LivraisonVue`)
- Modifier : `src-tauri/examples/ebook.rs` (lit `courant`/`provider`)
- Tests : `mod tests` de `projet.rs` et de `commands.rs`

**La règle de cette tâche : rien ne change à l'écran.** Les commandes gardent leurs noms
et leurs arguments (`destinataire_ajouter(provider_cle)`, …), et une `LivraisonVue` de
**compatibilité** sert au front la forme qu'il lit aujourd'hui — `destinataires`,
`provider` (la clé plate, retrouvée par `HERITEES`), `compose` recalculée. Le front et
les 9 fichiers de test JS ne bougent pas d'une ligne. C'est ce qui rend cette tâche
vérifiable seule : les deux suites vertes et le témoin inchangé prouvent que le
déplacement de la donnée est invisible.

C'est **un seul commit** : la forme de `Livraison` et la migration doivent atterrir
ensemble (sans `migre_livraison`, plus aucun `.ozalid` v4 ne s'ouvre), et `commands.rs`
suit parce que le crate compile en entier.

- [ ] **Étape 1 : les types, dans `projet.rs`**

`Destinataire` disparaît, `Livrable` prend sa place (verdict 2 pour la forme sérialisée —
`flatten` marche en TOML et en JSON, le `f64` traverse intact ; il désactive
`deny_unknown_fields`, qu'on ne pose donc pas) :

```rust
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
```

`Mesure` perd `dos` et gagne l'empreinte du gabarit — le doc-commentaire du type porte
les deux raisons :

```rust
/// Ce qu'une composition mesure, et que le projet retient — par **gabarit d'intérieur**
/// (POD, format, reliure), jamais par livrable : la pagination ne dépend ni du papier ni
/// de la finition, et c'est ce partage qui rend la comparaison de deux papiers gratuite
/// (spec § 5). Le dos n'y est plus : il dépend du papier, il se **recalcule** à chaque
/// vue depuis la formule du papier du livrable qu'on regarde.
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub polices_introuvables: Vec<String>,
}
```

> Le doc-commentaire existant de `polices_introuvables` est conservé tel quel.

`Livraison` :

```rust
/// À qui le livre est destiné, et pour lequel de ces livrables on regarde.
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
    #[serde(default)]
    pub deja_compose: bool,
    /// Les mesures, par clé de gabarit (`Fabrication::cle_gabarit`). Une map et non une
    /// liste : deux entrées de même gabarit sont impossibles par construction.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub mesures: std::collections::BTreeMap<String, Mesure>,
}
```

> Le doc-commentaire de `deja_compose` est conservé tel quel. Celui du type reprend la
> phrase existante (« Une seule liste et un pointeur dessus… »).

`Default` ne passe plus par la vue plate :

```rust
/// Un livre naît avec un livrable : le premier POD fourni, son premier format, sa
/// première reliure composable, son premier papier — c'est l'entrée que la table plate
/// mettait en tête, et le pointeur ne doit jamais être vide.
impl Default for Livraison {
    fn default() -> Self {
        let pod = &crate::catalogue::pods()[0];
        let reliure = pod
            .reliures
            .iter()
            .find(|r| r.geometrie.is_some())
            .expect("le premier POD fourni a une reliure composable");
        let l = Livrable::pour(crate::catalogue::Fabrication {
            pod: pod.cle.clone(),
            format: pod.formats[0].cle.clone(),
            reliure: reliure.cle.clone(),
            papier: pod.papiers[0].cle.clone(),
        });
        Self {
            courant: l.cle(),
            livrables: vec![l],
            deja_compose: false,
            mesures: std::collections::BTreeMap::new(),
        }
    }
}
```

> ⚠ Ne pas « améliorer » l'ordre des fournis au passage : Lulu reste premier, et son
> `108x175` n'a de gouttière publiée que pour 151–400 pages — un livre neuf hors tranche
> refuse déjà de composer aujourd'hui, ce n'est pas une régression de ce lot
> (reconnaissance § 7).

Les méthodes :

```rust
impl Livraison {
    /// Le livrable visé, s'il y en a un.
    pub fn courant(&self) -> Option<&Livrable> {
        self.livrables.iter().find(|l| l.cle() == self.courant)
    }

    /// Oublie ce que toutes les compositions ont mesuré. [doc existant conservé]
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

    /// Remet la liste d'accord avec le catalogue. [même arbitrage qu'aujourd'hui :
    /// élaguer vaut mieux que refuser d'ouvrir]
    fn normalise(&mut self) {
        let mut vus = std::collections::BTreeSet::new();
        self.livrables.retain_mut(|l| {
            if crate::catalogue::resout(&l.fabrication).is_err() {
                // Le papier est le seul axe qui se remplace sans changer le gabarit :
                // les trois autres partis — ou la reliure plus outillée —, le livrable
                // ne désigne plus rien de composable.
                let Some(pod) = crate::catalogue::pod(&l.fabrication.pod) else {
                    return false;
                };
                l.fabrication.papier = pod.papiers[0].cle.clone();
                if crate::catalogue::resout(&l.fabrication).is_err() {
                    return false;
                }
                // La mesure du gabarit survit : le papier ne pagine pas.
            }
            vus.insert(l.cle())
        });
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
        self.mesures
            .retain(|cle, m| gabarits.get(cle).is_some_and(|e| m.empreinte.as_deref() == Some(e)));
        if self.livrables.is_empty() {
            *self = Self::default();
        } else if self.courant().is_none() {
            self.courant = self.livrables[0].cle();
        }
    }
}
```

- [ ] **Étape 2 : la migration, dans `projet.rs`**

`VERSION` passe à **5**. Dans `migre`, à l'intérieur du bloc `if version < VERSION`,
avant l'estampille de version, l'appel `migre_livraison(&mut v);` et la fonction — sur le
`toml::Value`, comme les deux migrations qui la précèdent, et pour la même raison : en
v5, plus aucune structure Rust ne sait lire un `destinataire`. Le verdict 3 l'a fait
tourner sur le Candide réel et sur un ancien à trois destinataires ; ceci en est la forme
au propre :

```rust
/// v4 → v5 : le destinataire devient un livrable, la mesure quitte le destinataire pour
/// la map des gabarits, `courant` devient une clé à quatre axes. La table `HERITEES`
/// donne le triplet de chaque clé plate ; un prestataire disparu est élagué, comme
/// `normalise` l'aurait fait. Rejouée sur son propre résultat, elle ne bouge rien :
/// `destinataires` absent, elle sort au premier pas.
fn migre_livraison(v: &mut toml::Value) {
    let Some(l) = v.get_mut("livraison").and_then(toml::Value::as_table_mut) else {
        return;
    };
    let Some(anciens) = l.remove("destinataires") else {
        return; // déjà en v5, ou jamais de livraison écrite
    };
    let ancien_courant = l
        .get("courant")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    l.remove("courant");
    let Some(anciens) = anciens.as_array() else {
        return;
    };
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
```

- [ ] **Étape 3 : la couche de compatibilité, dans `commands.rs`**

`vise` — le point de passage unique, sans `&'static` (verdict 1c : le correctif aval est
neuf `&` et deux `.clone()`) :

```rust
/// Le livrable visé, résolu : sa vue plate, son papier, et le livrable lui-même.
fn vise(o: &Ouvert) -> Result<(Provider, catalogue::Papier, &Livrable), String> {
    let l = o
        .projet
        .meta
        .livraison
        .courant()
        .ok_or("aucun livrable : en déclarer un à l'étape Livraison.")?;
    let r = catalogue::resout(&l.fabrication)?;
    Ok((r.provider(), r.papier.clone(), l))
}
```

Sur les neuf appelants de `vise`, le motif est celui mesuré par la reconnaissance :
`let (pr, papier, d) = vise(o)?;` reste, et chaque usage aval qui attendait `&Provider`
reçoit `&pr` (huit erreurs E0308 attendues au premier `cargo check`, toutes de cette
famille — les corriger une à une, aucun combat d'emprunts n'est à prévoir). `papier`
change de même :

```rust
fn papier(pr: &Provider, cle: Option<&str>) -> Result<catalogue::Papier, String> {
    match cle {
        Some(c) => pr
            .papier(c)
            .cloned()
            .ok_or_else(|| format!("papier inconnu chez {} : {c}", pr.cle)),
        None => Ok(pr.papier_defaut().clone()),
    }
}
```

Les quatre commandes gardent **nom et arguments** ; elles traduisent la clé plate en
fabrication par `HERITEES` (rendre la table visible : `pub(crate)` l'est déjà) :

```rust
/// La fabrication qu'une clé plate désignait. Couche de compatibilité : elle meurt à la
/// tâche 6 avec le renommage des commandes.
fn fabrication_de(provider_cle: &str) -> Result<catalogue::Fabrication, String> {
    let (_, pod, format, reliure) = catalogue::HERITEES
        .iter()
        .find(|(h, ..)| *h == provider_cle)
        .ok_or_else(|| format!("prestataire inconnu : {provider_cle}"))?;
    let papier = catalogue::pod(pod)
        .ok_or_else(|| format!("POD inconnu : {pod}"))?
        .papiers[0]
        .cle
        .clone();
    Ok(catalogue::Fabrication {
        pod: (*pod).into(),
        format: (*format).into(),
        reliure: (*reliure).into(),
        papier,
    })
}

/// La clé plate d'un livrable, tant que le front la parle encore.
fn cle_plate(f: &catalogue::Fabrication) -> Option<&'static str> {
    catalogue::HERITEES
        .iter()
        .find(|(_, p, fo, r)| *p == f.pod && *fo == f.format && *r == f.reliure)
        .map(|(h, ..)| *h)
}
```

- `destinataire_ajouter(provider_cle)` : `fabrication_de` → `resout` → refus si un
  livrable de même **gabarit** existe déjà (« {libelle} est déjà destinataire de ce
  livre. ») → `livrables.push(Livrable::pour(f))`. Le refus porte sur le gabarit et non
  sur les quatre axes, **exprès** : l'écran d'aujourd'hui identifie ses lignes par la clé
  plate — deux livrables du même gabarit y produiraient deux `id` de DOM identiques. La
  règle à quatre axes arrive avec l'écran qui sait la porter, à la tâche 6.
- `destinataire_retirer(provider_cle)` : retire les livrables dont
  `cle_plate(&l.fabrication) == Some(&provider_cle)` ; refus du dernier et repositionnement
  de `courant` inchangés (le repositionnement pose `courant = livrables[0].cle()`).
- `destinataire_regler(destinataire: DestinataireCompat)` — le type `Destinataire`
  n'existant plus, la commande le redéclare pour elle seule :

```rust
/// Ce que l'écran envoie encore : la forme du destinataire d'avant les livrables.
/// Meurt à la tâche 6. C'est lui que le test du snake_case (commands.rs:2141) lit.
#[derive(Deserialize)]
pub struct DestinataireCompat {
    pub provider: String,
    pub papier: String,
    #[serde(default)]
    pub dos_mm: Option<f64>,
    #[serde(default)]
    pub fond_perdu_mm: Option<f64>,
}
```

  Le corps : retrouver le livrable par `cle_plate`, poser `fabrication.papier`,
  `dos_mm`, `fond_perdu_mm`, valider par `resout`, et **ne plus effacer de mesure** —
  le papier ne pagine pas, et le dos affiché se recalcule à la vue. (Changement de
  comportement voulu, c'est le cœur de la spec § 5 : régler un papier ne coûte plus une
  recomposition.) Si le livrable visé change de papier, `courant` suit :
  `if etait_courant { l.courant = livrable.cle(); }`.
- `destinataire_viser(provider_cle)` : retrouve le livrable par `cle_plate`, pose
  `courant = l.cle()`.

`composer` : `retenir_mesure` prend la clé de gabarit — qui **est** `pr.cle` depuis
`Resolu::provider()` — et la mesure porte l'empreinte ; le dos rendu à l'écran se calcule
comme aujourd'hui :

```rust
    let (pr, papier, _) = vise(o)?;
    let empreinte = {
        let l = o.projet.meta.livraison.courant().expect("vise vient de le trouver");
        catalogue::resout(&l.fabrication)?.empreinte()
    };
    ...
    let dos = papier.dos.mm(r.pages);
    o.projet.meta.livraison.retenir_mesure(
        &pr.cle,
        Mesure {
            pages: r.pages,
            gouttiere: r.gouttiere,
            blanche: r.blanche,
            empreinte: Some(empreinte),
            polices_introuvables: polices_introuvables.clone(),
        },
    );
```

> Le reste de `composer` est inchangé : `sorties_dossier(o, &pr.cle)` et
> `interieur_pdf(&dossier, &pr.cle)` portent désormais la clé de gabarit — le PDF de
> travail déménage de `sorties/bod/` vers `sorties/bod-135x215-broche/`. Un PDF composé
> avant ce commit reste orphelin sur le disque : sans gravité, la mesure migrée est
> périmée par l'empreinte absente et la recomposition l'écrira au nouvel endroit.

`vue` — le lien du pied suit la même clé, depuis la mesure du gabarit :

```rust
    let interieur_pdf = o
        .projet
        .meta
        .livraison
        .courant()
        .map(|l| l.fabrication.cle_gabarit())
        .filter(|g| o.projet.meta.livraison.mesure(g).is_some())
        .and_then(|g| {
            let dossier = sorties_dossier(o, &g).ok()?;
            let pdf = interieur_pdf(&dossier, &g);
            pdf.is_file().then(|| pdf.to_string_lossy().into_owned())
        });
```

La `LivraisonVue` de compatibilité, servie par `ProjetVue.livraison` (le champ change de
type, `Livraison` → `LivraisonVue`) :

```rust
/// Ce que l'écran lit de la livraison. Vue et non donnée : `compose` y est recalculée
/// par livrable depuis la mesure de son gabarit et la formule de son papier — le
/// déplacement de la mesure est invisible au front (décision du 26/08).
/// Forme de compatibilité jusqu'à la tâche 6 : les clés plates, `destinataires`.
#[derive(Serialize)]
pub struct LivraisonVue {
    destinataires: Vec<DestinataireVue>,
    courant: String,
    deja_compose: bool,
}

#[derive(Serialize)]
pub struct DestinataireVue {
    provider: String,
    papier: String,
    dos_mm: Option<f64>,
    fond_perdu_mm: Option<f64>,
    compose: Option<MesureVue>,
}

#[derive(Serialize)]
pub struct MesureVue {
    pages: u32,
    gouttiere: f64,
    blanche: bool,
    /// Recalculé ici, jamais retenu : c'est ce qui laisse deux papiers partager une
    /// mesure, et un `dos` de formule corrigée se corriger tout seul à la vue.
    dos: Option<f64>,
    polices_introuvables: Vec<String>,
}

fn livraison_vue(l: &Livraison) -> LivraisonVue {
    let vue = |liv: &Livrable| -> Option<DestinataireVue> {
        let plate = cle_plate(&liv.fabrication)?;
        let compose = l.mesure(&liv.fabrication.cle_gabarit()).map(|m| {
            let dos = catalogue::resout(&liv.fabrication)
                .ok()
                .and_then(|r| r.papier.dos.mm(m.pages));
            MesureVue {
                pages: m.pages,
                gouttiere: m.gouttiere,
                blanche: m.blanche,
                dos,
                polices_introuvables: m.polices_introuvables.clone(),
            }
        });
        Some(DestinataireVue {
            provider: plate.to_string(),
            papier: liv.fabrication.papier.clone(),
            dos_mm: liv.dos_mm,
            fond_perdu_mm: liv.fond_perdu_mm,
            compose,
        })
    };
    LivraisonVue {
        destinataires: l.livrables.iter().filter_map(&vue).collect(),
        courant: l
            .courant()
            .and_then(|liv| cle_plate(&liv.fabrication))
            .unwrap_or_default()
            .to_string(),
        deja_compose: l.deja_compose,
    }
}
```

`packager` : les `Cible` viennent maintenant de `resout` — clé de livrable, clé de
gabarit — et la mémoïsation de la tâche 3 devient effective :

```rust
    for d in &livrables {
        match catalogue::resout(&d.fabrication) {
            Ok(r) => cibles.push(package::Cible {
                pr: r.provider(),
                papier: r.papier.clone(),
                releve: planche::Releve { dos: d.dos_mm, fond_perdu: d.fond_perdu_mm },
                cle: d.cle(),
                cle_gabarit: d.fabrication.cle_gabarit(),
            }),
            Err(e) => sorties.push(Resultat {
                provider: d.cle(),
                libelle: d.cle(),
                package: None,
                vignette: None,
                erreur: Some(e),
            }),
        }
    }
```

(le `Resultat` garde son champ `provider` jusqu'à la tâche 6 ; il sert `d.cle()` — le
front ne le lit pas, vérifié par grep.) Les répertoires de package deviennent donc
`<projet>/bod-135x215-broche-creme-90/` : c'est la spec § 4, dès cette tâche.

`ebook_generer` et les commandes d'envois : mêmes `&pr` mécaniques ; `assembler_envois`
reçoit `&d.cle()`.

`examples/ebook.rs` : l'argument optionnel `<clé>` (« là que pour en essayer un autre »)
**disparaît** — le gabarit vient du livrable visé, comme dans l'application :

```rust
    let d = projet
        .meta
        .livraison
        .courant()
        .ok_or("aucun livrable dans ce projet.")?;
    let pr = catalogue::resout(&d.fabrication)?.provider();
    ...
    let r = ebook::generer(&projet, &pr, d.dos_mm, &PathBuf::from(&sortie), &typst)?;
```

(la ligne `let cle = args.next();` et son commentaire partent avec lui ; l'usage en
entête suit).

- [ ] **Étape 4 : réécrire les tests condamnés, écrire les neufs**

Dans `projet.rs`, la quinzaine du § 6 de la reconnaissance. Les remplaçants, avec leur
intention (chacun vu rouge : écrits avant l'implémentation qui les concerne, ou
vérifiés par mutation — par exemple `retenir_mesure` qui n'exigerait plus le gabarit
déclaré) :

```rust
#[test]
fn un_projet_sans_section_livraison_prend_le_premier_gabarit() {
    // (forme actuelle conservée) — attendus :
    // courant == "lulu-108x175-broche-standard"
    // livrables[0].fabrication.cle_gabarit() == "lulu-108x175-broche"
}

#[test]
fn la_liste_des_livrables_survit_a_l_aller_retour() {
    // deux livrables bod (creme-90) et coollibri-148x210 (mesure), finition Some("mat")
    // sur le premier : relus identiques, finition comprise.
}

#[test]
fn la_mesure_d_un_gabarit_survit_a_l_aller_retour() {
    // retenir_mesure("bod-135x215-broche", mesure avec empreinte réelle du catalogue),
    // écrire, relire : mesures["bod-135x215-broche"] égale, deja_compose vrai.
    // ⚠ l'empreinte doit être `resout(...).empreinte()` réelle, sinon `normalise`
    // la périme à la relecture — c'est le comportement voulu, pas un piège du test.
}

#[test]
fn une_mesure_est_partagee_par_les_livrables_du_meme_gabarit() {
    // bod creme-90 et bod blanc d'essai ne peuvent pas exister (bod n'a qu'un papier
    // aujourd'hui) : prendre kdp-6x9 creme et kdp-6x9 blanc. Une mesure retenue sous
    // "kdp-6x9-broche" est vue depuis les deux livrables (via `mesure(cle_gabarit)`),
    // et un gabarit tiers (lulu) ne la voit pas.
    // Remplace `une_mesure_ne_renseigne_que_son_destinataire` : l'invariant a changé
    // de sens, et ce test-ci est le nouveau sens.
}

#[test]
fn ce_qui_pagine_efface_toutes_les_mesures() {
    // inchangé dans l'esprit : modifier_livre / modifier_interieur / remplacer_texte
    // vident `mesures` ; deja_compose survit.
}

#[test]
fn perimer_une_mesure_n_efface_pas_l_histoire_du_livre() { /* idem, sur la map */ }

#[test]
fn une_livraison_incoherente_est_elaguee_plutot_que_refusee() {
    // un livrable au POD "disparu" : élagué ; les valides restent ; courant rattrapé.
}

#[test]
fn un_papier_disparu_retombe_sur_le_defaut_sans_perdre_la_mesure() {
    // livrable kdp-6x9 papier "nacre-introuvable" + mesure (empreinte réelle) sous
    // "kdp-6x9-broche" : après normalise, papier == "creme" (premier de kdp), la
    // mesure est toujours là. C'est le comportement neuf : le papier ne pagine pas.
}

#[test]
fn une_mesure_sans_empreinte_est_perimee_a_l_ouverture() {
    // mesure retenue avec empreinte None (le cas d'un .ozalid migré) : après
    // normalise, mesures vide, deja_compose toujours vrai — « dos périmé », pas
    // « jamais composé ».
}

#[test]
fn un_gabarit_reecrit_perime_la_mesure_a_l_ouverture() {
    // mesure avec empreinte "108x175|fausse|..." ≠ celle du catalogue : après
    // normalise, la mesure est partie. C'est la fragilité « gabarit réécrit » de la
    // spec § 8, fermée ici.
}

#[test]
fn une_reliure_non_outillee_est_elaguee_a_l_ouverture() {
    // livrable bod/135x215/rigide/creme-90 : élagué par normalise (resout la refuse),
    // spec § 9 par le Rust.
}

#[test]
fn une_livraison_videe_reprend_le_premier_gabarit() { /* forme actuelle, clés neuves */ }

#[test]
fn le_repli_de_police_survit_a_l_aller_retour() { /* sur la map des mesures */ }

#[test]
fn une_mesure_sans_le_champ_se_relit_vide() {
    // TOML littéral d'une mesure sans `polices_introuvables` ni `empreinte` : les
    // défauts jouent. (Littéral mis au format map : [livraison.mesures.kdp-6x9-broche])
}
```

Et la migration — les trois cas du verdict 3, en TOML littéraux dans les tests, à la
manière de `un_projet_v3_traverse_la_migration_sans_bouger` :

```rust
#[test]
fn un_projet_v4_migre_ses_destinataires_en_livrables() {
    // Le Candide réel : version = 4, [livraison] courant = "lulu",
    // [[livraison.destinataires]] provider = "lulu", papier = "standard".
    // Attendu : version 5, un livrable lulu/108x175/broche/standard,
    // courant == "lulu-108x175-broche-standard", mesures vide.
}

#[test]
fn la_migration_deplace_la_mesure_sous_le_gabarit() {
    // Trois destinataires : bod avec [destinataires.compose] (pages = 98, gouttiere,
    // blanche, dos = 7.21), kdp-55x85 avec dos_mm/fond_perdu_mm, et
    // "prestataire-disparu". Attendu : deux livrables, le disparu élagué, la mesure
    // sous "bod-135x215-broche" (l'ancien `dos` n'existe plus sur Mesure — il meurt à
    // la désérialisation), les relevés restés sur le livrable kdp, courant rattrapé.
    // NB : la mesure migrée n'a pas d'empreinte — après `normalise` elle est périmée ;
    // ce test lit donc le résultat de `migre` AVANT normalise, comme les tests v3 le
    // font, pour prouver le déplacement lui-même.
}

#[test]
fn la_migration_rejouee_ne_bouge_rien() {
    // migre(migre(v4)) == migre(v4) — l'idempotence du verdict 3.
}

#[test]
fn un_projet_v4_complet_traverse_la_migration() {
    // le jumeau v4→v5 de `un_projet_v3_traverse_la_migration_sans_bouger` : un
    // Metadonnees complet écrit en v4, relu en v5, tout le reste (livre, couverture,
    // interieur, envois) au caractère près.
}
```

Dans `commands.rs` : `le_destinataire_de_l_interface_se_lit` (2141) vise désormais
`DestinataireCompat` — il continue de prouver le snake_case de Tauri, même JSON littéral ;
`un_releve_absent_reste_absent` (2159) de même. Ils seront réécrits une seconde fois à la
tâche 6 contre `Livrable` — c'est prévu, pas un oubli.

- [ ] **Étape 5 : vérifier, largement**

`cargo test` complet (attendu : ~470+, 0 échec), `node --test tests/*.test.js` **sans y
avoir touché** (247 passés — c'est la preuve que la couche de compatibilité tient), le
témoin : **98 pages, dos 7,21 mm**.

- [ ] **Étape 6 : Commit**

« Le projet porte des livrables, et la mesure vit sous son gabarit d'intérieur »

---

### Tâche 5 : La vue plate change de clé

**Fichiers :**
- Modifier : `src-tauri/src/catalogue.rs` (`Provider.fabrication`, `aplatit`, `provider()` relégué, test d'ancrage converti)
- Modifier : `src-tauri/src/commands.rs` (compat servie en clés de gabarit, `ProviderVue`)
- Modifier : `src-tauri/tests/catalogue_initialise.rs`
- Modifier : `src-tauri/examples/temoin.rs`, `composer.rs`, `packager.rs`
- Modifier : `tests/*.test.js` (valeurs de fixtures seulement — pas les formes)

La clé de la vue plate cesse d'être la clé héritée : elle devient la clé de gabarit
(`lulu-108x175-broche`), la même que `Resolu::provider()` fabrique depuis la tâche 1 — et
le test d'ancrage de la tâche 1 (`le_livrable_resolu_fabrique_le_provider_de_la_vue_plate`)
gagne alors sa dernière assertion : les clés aussi sont égales.

- [ ] **Étape 1 : `Provider` porte sa fabrication, `aplatit` change de clé**

`Provider` gagne un champ (et `aplatit` comme `Resolu::provider()` le remplissent) :

```rust
pub struct Provider {
    pub cle: String, // la clé de GABARIT : pod-format-reliure
    /// La fabrication par défaut de cette entrée plate : son triplet, et le premier
    /// papier du POD. C'est elle que l'écran renvoie quand on ajoute depuis la liste.
    pub fabrication: crate::catalogue::Fabrication,
    ... // le reste inchangé
}
```

Dans `aplatit` :

```rust
            let fabrication = Fabrication {
                pod: pod.cle.clone(),
                format: f.cle.clone(),
                reliure: r.cle.clone(),
                papier: pod.papiers[0].cle.clone(),
            };
            v.push(Provider {
                cle: fabrication.cle_gabarit(),
                fabrication,
                ... // inchangé — `cle_heritee` n'est plus lue ici
            });
```

`Resolu::provider()` pose `fabrication` avec **son** papier (pas le défaut). Compléter le
test d'ancrage de la tâche 1 : `assert_eq!(fait.cle, plat.cle);`. Et honorer la note de
la tâche 3 : le refus de pagination de `package::assembler` cite désormais la reliure —
`… que {} accepte en {}.` avec `pr.libelle` et `pr.fabrication.reliure` (spec § 7), en
adaptant le test qui porte ce message s'il en fige le texte.

- [ ] **Étape 2 : `provider()` se relègue aux tests**

Plus aucun code de production n'appelle `provider(cle)` après la tâche 4 (le vérifier :
`grep -n "catalogue::provider(" src-tauri/src src-tauri/examples -r` ne doit montrer que
des tests et les exemples traités à l'étape 4 ci-dessous). Le helper devient :

```rust
/// Le provider d'une clé **plate historique** — helper de test, hors de l'application.
///
/// 76 tests de `interieur`, `planche`, `package`, `ebook` et `maquettes` nomment leurs
/// gabarits par la clé d'avant les livrables (`"bod"`, `"kdp-55x85"`). Plutôt que de
/// réécrire 76 ancrages qui ne testent pas l'identité, la traduction vit ici, sur la
/// même table que la migration.
#[cfg(test)]
pub fn provider(plate: &str) -> Option<&'static Provider> {
    let (_, pod, format, reliure) = HERITEES.iter().find(|(h, ..)| *h == plate)?;
    let gabarit = format!("{pod}-{format}-{reliure}");
    providers().iter().find(|p| p.cle == gabarit)
}
```

`cargo test --lib` doit rester à son compte : les 76 tests passent sans une ligne changée.
Si `la_source_porte_le_gabarit_du_prestataire_et_le_marqueur` (`interieur.rs:778`) fige la
clé en littéral (`"bod"`), il devient le seul à retoucher : la source Typst porte
désormais `bod-135x215-broche` — adapter l'assertion, c'est un changement voulu (le
commentaire d'entête du PDF nomme le gabarit).

- [ ] **Étape 3 : la compat de `commands.rs` parle en clés de gabarit**

Supprimer `fabrication_de`/`cle_plate` au profit de la vue plate elle-même — le front
envoie ce que `providers_liste` lui a donné, c'est-à-dire dorénavant la clé de gabarit :

```rust
fn plat(cle: &str) -> Result<&'static Provider, String> {
    catalogue::providers()
        .iter()
        .find(|p| p.cle == cle)
        .ok_or_else(|| format!("prestataire inconnu : {cle}"))
}
```

- `destinataire_ajouter` : `plat(&provider_cle)?.fabrication.clone()` → `resout` → push.
- `destinataire_retirer` / `viser` / `regler` : retrouvent le livrable par
  `l.fabrication.cle_gabarit() == provider_cle`.
- `livraison_vue` : `provider: liv.fabrication.cle_gabarit()` — plus de table inverse.
- `ProviderVue` sert la nouvelle clé sans changer de forme (il copie `p.cle`).

- [ ] **Étape 4 : fixtures JS, exemples, test d'intégration**

- Tests JS : **les valeurs seulement**. `'lulu'` → `'lulu-108x175-broche'`,
  `'kdp-6x9'` → `'kdp-6x9-broche'`, `'coollibri-148x210'` → `'coollibri-148x210-broche'`,
  etc., dans les fixtures `providers` et `PROJET` des 9 fichiers (`grep -n "provider"
  tests/*.test.js` donne la liste ; `placement.test.js` n'a rien). Les champs gardent
  leurs noms : c'est ce qui évite la boucle infinie du verdict § 6. `node --test` doit
  rendre la main **et** être vert.
- `examples/temoin.rs` :

```rust
const FABRICATION: (&str, &str, &str, &str) = ("bod", "135x215", "broche", "creme-90");
...
    let (pod, format, reliure, papier) = FABRICATION;
    let r = catalogue::resout(&catalogue::Fabrication {
        pod: pod.into(),
        format: format.into(),
        reliure: reliure.into(),
        papier: papier.into(),
    })?;
    let pr = r.provider();
    let int = package::composer_interieur(&projet, &pr, &pr.cle, &sortie, &typst)?;
    let p = package::assembler(
        &projet, &pr, r.papier, Releve::default(), &pr.cle, &int, &sortie, &typst,
    )?;
```

  (le commentaire d'entête « Le gabarit est `bod` » devient « Le gabarit est BoD » avec le
  triplet ; `PAGES_ATTENDUES` ne bouge pas — c'est le point du témoin.)
- `examples/composer.rs` et `examples/packager.rs` : l'argument `<prestataire>` devient
  `<pod> <format> <reliure>` (usage mis à jour), résolu par `resout` avec le premier
  papier du POD.
- `src-tauri/tests/catalogue_initialise.rs` : les assertions par clé passent aux clés de
  gabarit, et le test vérifie de surcroît que `pods()` voit le POD déposé (le
  `OnceLock` `PODS` est posé par `initialiser` depuis la tâche 1 — c'est ici qu'on le
  prouve sur le vrai chemin de démarrage). Toujours **un seul** `#[test]`.

- [ ] **Étape 5 : vérifier puis Commit**

Suites complètes + témoin. Commit :
« La vue plate se nomme par son gabarit, la clé héritée ne sert plus qu'aux tests »

---

### Tâche 6 : Les commandes `livrable_*` et le front

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs` (renommages, `LivrableVue` définitive, `Resultat.cle`, tests 2141/2159 réécrits, refus du doublon)
- Modifier : `src-tauri/src/lib.rs` (`generate_handler`, lignes 126–129)
- Modifier : `src/app.js`, `src/livraison.js`, `src/couverture.js`, `src/envois.js`
- Modifier : `tests/coquille.test.js`, `packages.test.js`, `contrats.test.js`, `composition.test.js`, `couverture.test.js`, `cycle_de_vie.test.js`, `epreuve.test.js`, `ebook.test.js`, `dom_shim.test.js`

Le flip atomique : Rust et JS changent ensemble, en un commit, parce que la forme des
commandes et celle de la vue sont un seul contrat (`contrats.test.js` existe pour ça).

- [ ] **Étape 1 : les commandes, côté Rust**

```rust
/// Ajoute un livrable au livre.
///
/// Le refus du doublon porte sur les **quatre axes de fabrication** : deux livrables qui
/// ne différeraient que par la finition produiraient les mêmes octets dans deux
/// répertoires (spec § 4) — la finition est une donnée de commande, pas de fabrication.
#[tauri::command]
pub fn livrable_ajouter(
    fabrication: catalogue::Fabrication,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    let r = catalogue::resout(&fabrication)?;
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let l = &mut o.projet.meta.livraison;
    let cle = fabrication.cle();
    if l.livrables.iter().any(|x| x.cle() == cle) {
        return Err(format!(
            "{} en {} est déjà un livrable de ce livre — la finition seule n'en fait \
             pas un autre : le fichier produit serait le même.",
            r.pod.nom, r.papier.nom
        ));
    }
    l.livrables.push(Livrable::pour(fabrication));
    vue_modifiee(o)
}

#[tauri::command]
pub fn livrable_retirer(cle: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    // corps de `destinataire_retirer`, la recherche sur `x.cle() == cle` ;
    // le refus du dernier et le repositionnement de `courant` inchangés.
}

/// Le papier, la finition et les relevés d'un livrable. `cle` désigne le livrable tel
/// qu'il était : changer son papier change son identité, et `courant` suit.
#[tauri::command]
pub fn livrable_regler(
    cle: String,
    livrable: Livrable,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    let r = catalogue::resout(&livrable.fabrication)?;
    if let Some(f) = &livrable.finition {
        if !r.pod.finitions.iter().any(|x| &x.cle == f) {
            return Err(format!("finition inconnue chez {} : {f}.", r.pod.nom));
        }
    }
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let l = &mut o.projet.meta.livraison;
    let neuve = livrable.cle();
    if neuve != cle && l.livrables.iter().any(|x| x.cle() == neuve) {
        return Err(format!("{neuve} est déjà un livrable de ce livre."));
    }
    let place = l
        .livrables
        .iter_mut()
        .find(|x| x.cle() == cle)
        .ok_or_else(|| format!("{cle} n'est pas un livrable de ce livre."))?;
    // Seuls le papier, la finition et les relevés se règlent sur une ligne : le gabarit
    // ne bouge pas, donc la mesure non plus — le dos affiché se recalcule à la vue.
    if place.fabrication.cle_gabarit() != livrable.fabrication.cle_gabarit() {
        return Err("le gabarit d'un livrable ne se règle pas : retirer, puis ajouter.".into());
    }
    *place = livrable;
    if l.courant == cle {
        l.courant = neuve;
    }
    vue_modifiee(o)
}

#[tauri::command]
pub fn livrable_viser(cle: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    // corps de `destinataire_viser`, la recherche sur `x.cle() == cle`.
}
```

`fabrication_de`/`plat`/`DestinataireCompat` : supprimés (la couche de compatibilité meurt
ici — `plat` reste si `livrable_ajouter` côté front continue de passer par la liste, voir
étape 3 : le front envoie la `fabrication` entière, `plat` n'est donc plus nécessaire).
`lib.rs` : les quatre entrées du `generate_handler` deviennent `commands::livrable_*`.

La vue définitive :

```rust
#[derive(Serialize)]
pub struct LivraisonVue {
    livrables: Vec<LivrableVue>,
    /// La clé du livrable visé — quatre axes.
    courant: String,
    deja_compose: bool,
}

#[derive(Serialize)]
pub struct LivrableVue {
    /// L'identité à quatre axes : l'identifiant des lignes, des DOM et des commandes.
    /// Fabriquée par le Rust et servie telle quelle — jamais recomposée côté JS
    /// (deux fabricants finiraient par diverger, verdict 5d).
    cle: String,
    /// La clé du gabarit : la jointure vers la liste des providers, et rien d'autre.
    gabarit: String,
    pod: String,
    format: String,
    reliure: String,
    papier: String,
    finition: Option<String>,
    dos_mm: Option<f64>,
    fond_perdu_mm: Option<f64>,
    compose: Option<MesureVue>,
}
```

(`MesureVue` inchangée ; `livraison_vue` remplit `cle: liv.cle()`,
`gabarit: liv.fabrication.cle_gabarit()`, les quatre axes depuis `liv.fabrication`, et
`courant: l.courant.clone()` — plus aucune traduction.) `Resultat.provider` devient
`Resultat.cle` (il sert déjà `d.cle()` depuis la tâche 4).

Tests `commands.rs` :

```rust
#[test]
fn le_livrable_de_l_interface_se_lit() {
    // Le successeur du test du snake_case : le JSON que Tauri livre, avec la
    // fabrication APLATIE par `#[serde(flatten)]` — c'est le contrat du front.
    let l: Livrable = serde_json::from_str(
        r#"{"pod":"kdp","format":"6x9","reliure":"broche","papier":"creme",
            "dos_mm":1.5,"fond_perdu_mm":null}"#,
    )
    .unwrap();
    assert_eq!(l.cle(), "kdp-6x9-broche-creme");
    assert_eq!(l.dos_mm, Some(1.5));
    assert!(l.finition.is_none());
}

#[test]
fn un_releve_absent_reste_absent() { /* même JSON, sans dos_mm : None. */ }
```

Et le refus du doublon, vu rouge d'abord (l'écrire avant le corps de `livrable_ajouter`
n'est pas possible ici — la commande prend un `State` ; le tester au niveau de la
`Livraison` : ajouter la garde de doublon est dans la commande, donc **mutation ciblée** :
commenter le `if` du refus et voir le test d'intégration JS de l'étape 4 échouer, ou
extraire la garde en fonction libre testable. Le plus simple qui reste honnête : une
fonction libre) :

```rust
/// Le refus d'un livrable déjà déclaré, à quatre axes — la finition n'y est pas.
fn refuse_doublon(livrables: &[Livrable], cle: &str) -> bool {
    livrables.iter().any(|x| x.cle() == cle)
}

#[test]
fn deux_livrables_identiques_sur_les_quatre_axes_sont_refuses() {
    let un = Livrable::pour(fabrication("kdp", "6x9", "broche", "creme"));
    let mut deux = un.clone();
    deux.finition = Some("mat".into());
    // La finition ne distingue pas : même clé, refusé.
    assert!(refuse_doublon(&[un], &deux.cle()));
}
```

- [ ] **Étape 2 : le front — `app.js`**

Renommages (chaque site listé par la reconnaissance § 2) :

- `destinataireCourant()` devient `livrableCourant()` :

```js
/** Le livrable visé : son papier, sa finition et ses relevés. */
function livrableCourant() {
  return projet?.livraison.livrables.find((d) => d.cle === projet.livraison.courant);
}
```

- `providerCourant()` joint par le gabarit :

```js
function providerCourant() {
  const d = livrableCourant();
  return providers.find((p) => p.cle === d?.gabarit);
}
```

- Le libellé d'un livrable dit son papier — deux livrables du même gabarit doivent se
  lire distincts dans le pied :

```js
/** Le libellé d'un livrable : son gabarit, et le papier qui le distingue. */
function libelleLivrable(d) {
  const p = providers.find((x) => x.cle === d.gabarit);
  const papier = p?.papiers.find((x) => x.cle === d.papier)?.libelle ?? d.papier;
  return `${p?.libelle ?? d.gabarit} — ${papier}`;
}
```

  (`libelleProvider(cle)` reste pour la liste d'ajout.) Le sélecteur du pied
  (`majPied`) : `new Option(libelleLivrable(d), d.cle)`, `sel.value = courant`.
- `dosPerime`, `repliPolices`, `dosCourant` : `destinataireCourant()` → `livrableCourant()`
  (la forme de `compose` n'a pas bougé — c'est la décision LivraisonVue).
- La **veille durcie** — le verdict § 6 a montré la boucle infinie, la garde la ferme :

```js
function veiller() {
  if (veilleSuspendue) {
    veilleSuspendue = false;
    return;
  }
  // Un `courant` qui ne désigne rien n'arme jamais la veille : recomposer sans
  // destinataire boucle sans fin, sans erreur — le Rust garantit l'invariant, cette
  // garde le tient si un état transitoire le casse.
  const c = livrableCourant();
  if (!(consenti || projet?.livraison.deja_compose) || !c || c.compose) return;
  clearTimeout(attenteComposition);
  attenteComposition = setTimeout(() => recomposer(false), DELAI_COMPOSITION);
}
```

  Même garde dans `recomposer` : `if (!force && (livrableCourant()?.compose ?? true)) return;`
  — un courant absent vaut « rien à recomposer ».
- Les écouteurs (`app.js:1259–1266`) :

```js
$('inDestinataire').addEventListener('change', () => tente(async () => {
  afficherProjet(await invoke('livrable_viser', { cle: $('inDestinataire').value }));
  oublierPages();
}));
$('btAjouterDestinataire').addEventListener('click', () => tente(async () => {
  const p = providers.find((x) => x.cle === $('inAjoutDestinataire').value);
  afficherProjet(await invoke('livrable_ajouter', {
    fabrication: { pod: p.pod, format: p.format, reliure: p.reliure, papier: p.papiers[0].cle },
  }));
}));
```

  — ce qui suppose `ProviderVue` enrichi : ajouter `pod`, `format`, `reliure` (les clés de
  `p.fabrication`) à `ProviderVue` et son `From<&Provider>` (Rust, même commit).

- [ ] **Étape 3 : le front — `livraison.js`, `couverture.js`, `envois.js`**

`livraison.js`, `afficherDestinataires()` : la boucle passe sur
`projet.livraison.livrables`, et **tous les identifiants de DOM prennent `d.cle`**
(`dest-papier-${d.cle}`, `dest-retirer-${d.cle}`, `dest-${quoi}-${d.cle}`) — unique par
construction, valide en `id` puisque toutes les clés sont des noms (tâche 2). La ligne :

```js
  for (const d of projet.livraison.livrables) {
    const p = providers.find((pr) => pr.cle === d.gabarit);
    ...
    ligne.append(h('span', libelleProvider(d.gabarit), 'nom'));
    // le select de papier : value = d.papier, change → reglerLivrable(d)
    // les relevés : champReleve(`dest-${quoi}-${d.cle}`, …, d.cle)
    // retirer : invoke('livrable_retirer', { cle: d.cle })
```

`reglerDestinataire(cle)` devient `reglerLivrable(d)` — il renvoie le livrable entier,
avec son identité d'avant :

```js
/** Relit la ligne d'un livrable et la renvoie au projet. */
async function reglerLivrable(d) {
  const lu = (id) => {
    const v = $(id)?.value.trim();
    return v ? Number(v) : null;
  };
  await tente(async () => afficherProjet(await invoke('livrable_regler', {
    cle: d.cle,
    livrable: {
      pod: d.pod, format: d.format, reliure: d.reliure,
      papier: $(`dest-papier-${d.cle}`).value,
      finition: d.finition ?? null,
      dos_mm: lu(`dest-dos-${d.cle}`),
      fond_perdu_mm: lu(`dest-fp-${d.cle}`),
    },
  })));
}
```

La liste d'ajout **ne filtre plus** les gabarits déjà déclarés : c'est ce qui permet de
déclarer BoD deux fois pour comparer deux papiers — le Rust refuse le vrai doublon (les
quatre axes) avec sa raison. `restants` disparaît, le select liste `providers` entier :

```js
  const sel = $('inAjoutDestinataire');
  sel.replaceChildren();
  for (const p of providers) sel.append(new Option(p.libelle, p.cle));
  sel.disabled = providers.length === 0;
  $('btAjouterDestinataire').disabled = providers.length === 0;
```

`couverture.js` (`formatCourant`, ligne ~833) et `envois.js` (`teintePapier`, ligne ~367) :
la jointure passe par `d.gabarit` et `d.papier` :

```js
function formatCourant() {
  const d = livrableCourant();
  const p = providers.find((x) => x.cle === d?.gabarit);
  return p ? { largeur: p.largeur, hauteur: p.hauteur } : null;
}
```

```js
function teintePapier() {
  const l = projet?.livraison;
  const d = l?.livrables.find((x) => x.cle === l.courant);
  const pr = providers.find((p) => p.cle === d?.gabarit);
  const pa = pr?.papiers.find((x) => x.cle === d?.papier) ?? pr?.papiers[0];
  return pa?.teinte ?? '#ffffff';
}
```

- [ ] **Étape 4 : les 9 fichiers de test JS**

Un seul mouvement par fichier : le faux Rust **et** ses lecteurs (verdict § 6 — jamais
l'un sans l'autre). Concrètement :

- `coquille.test.js` (76 tests) et `packages.test.js` (36) — les deux faux Rust :
  `livraison.destinataires` → `livraison.livrables`, chaque fixture de livrable gagne
  `cle` (fabriquée une fois dans le helper, pas recomposée dans chaque test),
  `gabarit`, `pod`/`format`/`reliure` ; `chez(p)` devient :

```js
const chez = (p) => ({
  cle: `${p.pod}-${p.format}-${p.reliure}-${p.papiers[0].cle}`,
  gabarit: p.cle, pod: p.pod, format: p.format, reliure: p.reliure,
  papier: p.papiers[0].cle, finition: null, dos_mm: null, fond_perdu_mm: null,
  compose: null,
});
```

  et les commandes simulées suivent le contrat neuf : `livrable_viser` (`args.cle`),
  `livrable_regler` (`args.cle`, `args.livrable` — le faux applique le papier et **ne
  touche plus à `compose`**), `livrable_ajouter` (`args.fabrication`),
  `livrable_retirer` (`args.cle`). Les assertions sur les appels
  (`dernier(appels, 'destinataire_retirer')[1].providerCle`) suivent.
- Les 7 autres : la fixture `PROJET` prend la forme neuve (livrables + `cle` + `courant`
  à quatre axes) ; `contrats.test.js` remplace ses quatre lignes `destinataire_*` par les
  contrats `livrable_*`.
- Les fixtures de `packager` : `provider:` → `cle:` dans les `Resultat` simulés.
- **Test neuf, vu rouge d'abord** (le rouge : retirer provisoirement la garde `!c` de
  `veiller` et le voir appeler `composer`) — dans `coquille.test.js`, à côté des tests de
  veille existants :

```js
test('un courant qui ne désigne aucun livrable n’arme pas la veille', async () => {
  // un projet dont courant pointe un livrable retiré : la veille ne doit pas armer —
  // sans la garde, elle recompose en boucle sans erreur (reconnaissance § 6).
  ...suivre le montage des tests de veille voisins : projet avec deja_compose = true,
  livraison.courant = 'fantome-108x175-broche-standard', livrables = [chez(LULU)],
  déclencher veiller(), avancer le débounce, et vérifier qu'aucun appel `composer`
  n'est parti (le journal `appels` des coquilles le porte déjà)...
});
```

  (S'aligner sur l'outillage réel du fichier — faux timers ou attente courte — plutôt que
  d'en inventer un ; les tests de veille voisins montrent le geste exact.)

- [ ] **Étape 5 : vérifier**

`node --test tests/*.test.js` : la suite **rend la main** et elle est verte — si elle ne
rend pas la main, c'est la boucle du verdict § 6 : un lecteur et un faux Rust désaccordés.
`cargo test` complet. Témoin inchangé. Puis, à la main si l'environnement le permet,
`cargo run` et l'étape Livraison : ajouter KDP 6×9 deux fois, changer le papier de la
première ligne — deux lignes distinctes, le refus du vrai doublon en clair.

- [ ] **Étape 6 : Commit**

« Les commandes parlent en livrables, et l'écran distingue deux papiers d'un même gabarit »

---

### Tâche 7 : La clé héritée disparaît

**Fichiers :**
- Modifier : `src-tauri/pods/*.toml` (14 lignes)
- Modifier : `src-tauri/src/catalogue.rs` (`Format`, `verifie`, `charge`)
- Modifier : `src-tauri/tests/catalogue_initialise.rs` (si une assertion la lit encore)

- [ ] **Étape 1 : la retirer partout**

- Les six fichiers `src-tauri/pods/*.toml` : supprimer les 14 lignes `cle_heritee = "…"`.
- `Format` : supprimer le champ `cle_heritee` (avec `deny_unknown_fields`, un fichier du
  poste écrit pour le lot 1 qui la porterait encore sera **refusé en la nommant** — c'est
  le comportement voulu : personne n'a déposé de surcharge à ce jour, et le refus dit
  quoi retirer).
- `verifie` : supprimer le contrôle `est_un_nom(&f.cle_heritee)` et le `sans_doublon` des
  clés héritées.
- `charge` : supprimer le bloc de collision inter-POD des clés héritées (la clé de
  gabarit est préfixée par la clé du POD : la collision inter-POD est impossible par
  construction, et le remplacement même-clé la couvre au sein d'un POD).
- Le test `chaque_cle_heritee_a_son_triplet` (tâche 1) perd sa seconde assertion
  (`r.format.cle_heritee`) et son commentaire dit ce qu'il reste : la table de migration
  résout, et elle a quatorze entrées — c'est l'ancrage des `.ozalid` anciens, il ne
  disparaîtra qu'avec la migration v5 elle-même.

- [ ] **Étape 2 : prouver le vide**

`grep -rn "cle_heritee" src-tauri/ tests/ src/` → **zéro** occurrence hors
`docs/`. `cargo test` complet, `node --test`, témoin.

- [ ] **Étape 3 : Commit**

« La clé héritée quitte les fichiers : l'identité à quatre axes est seule »

---

### Tâche 8 : La spec rejoint les faits

**Fichiers :**
- Modifier : `docs/superpowers/specs/2026-08-26-catalogue-et-livrables-design.md`

Les trois corrections relevées par la reconnaissance, plus la décision d'arbitrage :

- [ ] § 3 : « les deux seules signatures `&'static Provider` hors tests —
  `commands.rs:467` et `commands.rs:1890` » → la fonction s'appelle **`vise`** (pas
  `couple`), elle était aux lignes **479 et 1904**, et le lot 2 a retiré ces `&'static`
  (verdict 1c) — reformuler au passé.
- [ ] § 4 : `bod-135x215-broche-creme90/` → **`bod-135x215-broche-creme-90/`**, avec une
  phrase qui fixe la règle : les quatre clés jointes par des tirets, telles quelles,
  jamais transformées, jamais re-découpées (décision du 26/08).
- [ ] § 8, risque « gabarit réécrit » : noter qu'il est **fermé** par l'empreinte de
  gabarit portée par la mesure et comparée à l'ouverture.
- [ ] Commit : « La spec rejoint le code : vise, cinq segments, l'empreinte »

---

## À l'œil, avant de clore le lot

Aucune de ces vérifications n'est automatisable ici ; elles closent le lot, sur un livre
réel (`build/travail/candide.ozalid` est le candidat naturel — **copie de travail
d'abord** : une fois enregistré, il sera en v5 et l'ancienne application ne le relira
plus, spec § 8).

1. **La migration** : ouvrir un `.ozalid` v4 — il s'ouvre sans un mot, la Livraison
   montre son destinataire d'hier (Lulu, papier Standard), enregistrer, rouvrir :
   toujours là, et le fichier est en v5.
2. **La comparaison de papiers** : déclarer KDP 6×9 crème **et** KDP 6×9 blanc (BoD n'a
   qu'un papier tant que le lot 4 ne l'a pas complété — la spec dit BoD, KDP est
   l'équivalent disponible), générer : **deux répertoires**
   (`kdp-6x9-broche-creme/`, `kdp-6x9-broche-blanc/`), deux dos différents dans les
   comptes rendus, et le second package copié, pas recomposé (les deux
   `interieur-*.pdf` identiques à l'octet — `cmp` fait foi).
3. **Une seule composition à l'écran** : entre les deux livrables ci-dessus, changer la
   visée dans le pied — aucune recomposition ne part (le pied garde ses chiffres, le dos
   change tout seul avec le papier).
4. **Le dépôt d'un POD du poste** : déposer un `.toml` dans
   `~/Library/Application Support/<app>/pods/`, relancer : le POD paraît dans la liste
   d'ajout sans recompilation. Y mettre une faute de frappe, relancer : l'application
   démarre, la Livraison nomme le fichier et la raison. (Ces deux-là restaient à faire
   du lot 1 — les faire ici une fois pour toutes, sur les clés neuves.)
5. **Le gabarit réécrit** : dans ce `.toml` du poste, changer une marge, relancer,
   rouvrir le livre : le pied dit « dos périmé » au lieu d'afficher la mesure d'hier.

## Ce que ce lot ne fait pas

- **La cascade et le grisé** (POD puis format à l'ajout, la reliure non outillée grisée
  avec sa raison) : lot 3. L'écran du lot 2 garde la liste plate — enrichie du refus
  Rust et de deux papiers comparables.
- **La finition n'a pas de contrôle à l'écran** : le champ existe, la commande le valide,
  le lot 3 lui donnera son réglage sur la ligne.
- **BoD complété et le COOKBOOK** : lot 4. Le COOKBOOK ment déjà (« `providers.rs` fait
  foi », verdict 5e) — dette assumée par la spec, qui grandit d'un lot : ne pas la
  corriger en passant ici.
- **La commande directe chez un POD** : question ouverte de la session du 26/08, sans
  suite décidée.
