# Catalogue lot 3 — la cascade

> **Pour un exécutant agentique :** SOUS-COMPÉTENCE REQUISE : `superpowers:subagent-driven-development`
> (recommandé) ou `superpowers:executing-plans` pour exécuter ce plan tâche par tâche. Les
> étapes sont des cases à cocher (`- [ ]`).

**But :** l'écran Livraison parle le catalogue à cinq axes. À l'ajout, deux listes en
cascade — le POD, puis **ses** formats. Sur la ligne posée, trois réglages — reliure,
finition, papier —, chacun limité à ce que ce POD offre, et une reliure non outillée
grisée **avec sa raison en clair sous elle**. Le vocabulaire visible passe de
« destinataire » à « livrable ».

**Architecture :** une commande d'arbre, `pods_liste`, **s'ajoute** à `providers_liste`
sans la remplacer — la table plate garde ses quatre lecteurs (`couverture.js`,
`envois.js`, deux fonctions d'`app.js`), qui joignent par la clé de gabarit et n'ont rien
à voir avec la cascade (reconnaissance, verdict 4). Le front tient donc deux tables : la
plate pour ce que la projection sait dire (format en mm, fond perdu effectif, libellé
composé), l'arbre pour ce qu'un POD offre. Le Rust ne gagne **aucune** garde : elles sont
toutes déjà là (verdict 1) ; il en perd une, le verrou sur la reliure (verdict 2).

**Pile :** Rust 2021, Tauri 2, `serde`. Front vanilla, sans bundler. Tests : `cargo test`
depuis `src-tauri/`, `node --test tests/*.test.js` depuis la racine,
`cargo run --example temoin` comme témoin.

**Spec :** `docs/superpowers/specs/2026-08-26-catalogue-et-livrables-design.md` (§ 6, et
§ 9 pour ce que les tests doivent tenir).
**Reconnaissance :** `docs/superpowers/2026-08-26-reconnaissance-lot-3.md` — les verdicts
cités (1 à 6) y sont, chacun avec ses `fichier:ligne`.

---

## Décisions arbitrées (utilisateur, 26/08) — ne pas les rouvrir

1. **La finition** : le contrôle est posé mais ne paraît que chez un POD qui déclare des
   finitions. Aucun POD livré n'en déclare (verdict 3) ; aucun relevé n'est fait dans ce
   lot — c'est le lot 4 qui remplira BoD.
2. **La reliure se règle sur la ligne**, conformément à la spec § 6. `reglage_refuse` ne
   verrouille plus que le couple (POD, format) — les deux axes que la cascade choisit à
   l'ajout.
3. **`dos_publie` passe du POD au papier.** Ce qui réclame un relevé de dos suit le papier
   réellement choisi, et non le premier de la liste (verdict, § 2 de la reconnaissance).
4. **Le renommage « destinataire » → « livrable » entre dans ce lot**, README compris, en
   **tâche séparée** : un renommage de ~160 points noyé dans la refonte de l'écran rend la
   revue de la cascade illisible.
   *Étendu le 26/08, après la tâche 7 : les 32 commentaires Rust et les sept passages de
   README hors de la section visée disaient encore « destinataire » là où le code dit
   « livrable » depuis le lot 2. 14 renommés, 12 laissés — ceux qui **narrent** la migration
   v4→v5, où le mot désigne la clé d'hier ; les clés TOML littérales `[[destinataires]]` des
   fixtures de migration restent, sous peine de casser la lecture des anciens fichiers.*
5. **Le libellé de ligne ne gagne rien** : `libelleProvider(d.gabarit)` porte déjà le POD
   et le format, et ces deux axes ne se règlent plus.
6. **`libelleLivrable` porte la reliure** en plus du papier : il sert le pointeur du pied
   et les comptes rendus de package, où aucun contrôle ne se lit, et deux livrables ne
   différant que par leur reliure s'y liraient identiques.
   *Affiné à la tâche 6 : elle n'y paraît que chez un POD qui en offre **plusieurs de
   composables**. Ailleurs elle ne distingue rien et coûte cher à lire — « Lulu — poche
   108 × 175 — Broché — dos carré collé — Papier standard », quatre tirets cadratins dont un
   interne au nom de la reliure. Un libellé dit ce qui distingue, pas tout ce qu'on sait ;
   sur le catalogue livré les libellés sont donc inchangés, et les trois tests qui les
   ancrent le prouvent.*

## Invariants sur lesquels ce plan s'appuie

- **`geometrie.is_some()` ⟺ `non_outille.is_none()`**, garanti par `verifie_reliure`
  (`src-tauri/src/catalogue.rs:466-493`) : les cas `(None, None)` et `(Some, Some)` sont
  refusés à la lecture. Le front peut donc décider « composable » sur le seul
  `non_outille === null`, sans que la géométrie ait à traverser la vue.
- **La garde du grisé existe déjà** : `catalogue::resout` refuse une reliure non outillée
  en rendant la raison du fichier (`catalogue.rs:786-791`), ancrée par le test
  `catalogue.rs:2106`. Le grisé de l'écran n'est que la **lecture** de ce refus ; il ne le
  remplace pas.
- **Un POD sans reliure composable n'existe pas dans le catalogue chargé** :
  `Pod::verifie` le refuse en nommant son fichier (`catalogue.rs:367-373`, testé
  l. 1683), précisément pour qu'un imprimeur ne disparaisse pas sans un mot. Le filtre
  qu'`aplatit` porte (`catalogue.rs:617-652`) — et celui que la tâche 1 reprend — ne peut
  donc pas se déclencher : il tient les deux projections d'accord, il ne rattrape rien.
  Griser un POD entier n'a par conséquent aucun objet, et serait une fonction que
  personne n'a demandée.

  *(Corrigé après la tâche 1 : la reconnaissance avait manqué ce refus et le plan
  l'attribuait à `aplatit`.)*

## Avant chaque commit

Valables pour **toutes** les étapes « Commit », sans être répétées. Depuis `src-tauri/` :

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Depuis la racine : `node --test tests/*.test.js`.

Et, tout fichier de `src-tauri/` ayant changé dans la tâche :
`cd src-tauri && cargo run --example temoin`. Attendu, à chaque tâche, sans exception :
**98 pages, dos 7,21 mm**. Un écart n'est pas à corriger dans le témoin : c'est le signe
qu'une valeur s'est perdue en route.

`cargo test -- --ignored` n'est pas requis dans ce lot : aucune tâche ne touche
`package.rs`, `interieur.rs`, `planche.rs` ni `typst.rs`. Si une tâche finit par y toucher,
le lancer (~1,3 s).

## Pièges transverses

- **Neuf fichiers de tests portent un faux `invoke` qui jette sur une commande
  inconnue** (`throw new Error('commande inattendue : …')`, `contrats.test.js:62`).
  Ajouter `pods_liste` au démarrage les casse **tous** tant qu'ils ne la connaissent pas :
  `composition`, `contrats`, `coquille`, `couverture`, `cycle_de_vie`, `dom_shim`,
  `ebook`, `epreuve`, `packages`. C'est la tâche 3 qui les complète, en un geste.
- **Un renommage JS incohérent ne fait pas échouer la suite, il la fait boucler** sans
  fin (piège relevé au lot 2). Si `node --test` ne rend pas la main en une minute, c'est
  ce bug-là : un faux Rust changé sans ses lecteurs, ou l'inverse.
- **Le front est embarqué dans le binaire à la compilation** : après un changement de
  `src/` seul, `touch src-tauri/src/lib.rs` avant `cargo build`, sinon le binaire garde
  l'ancien front.
- **Tout littéral Rust contenant un TOML de POD avec une `teinte` doit être `r##"…"##`** :
  `teinte = "#f7f0e0"` contient `"#`, qui ferme un `r#"…"#`.
- **La clé du POD TheBookEdition est `tbe`**, pas `thebookedition`.

## Structure des fichiers

| Fichier | Responsabilité dans ce lot |
|---|---|
| `src-tauri/src/commands.rs` | `PodVue` et ses quatre vues filles, la commande `pods_liste`, `dos_publie` déplacé sur `PapierVue`, `reglage_refuse` assoupli |
| `src-tauri/src/lib.rs` | `pods_liste` dans le `generate_handler` |
| `src/index.html` | les deux `<select>` de la cascade, le `<h2>`, les ids renommés |
| `src/livraison.js` | la cascade, les trois réglages de la ligne, le grisé motivé |
| `src/app.js` | `pods` chargé au démarrage, le bouton d'ajout, `dos_publie` par papier au pied, `libelleLivrable` |
| `src/styles.css` | la disposition d'une ligne à quatre contrôles, la raison sous l'option grisée |
| `tests/*.test.js` | `pods_liste` dans neuf faux ; les tests de la cascade, du grisé et du réglage de reliure ; les ids renommés |
| `README.md` | § « Le prestataire, choisi une seule fois » réécrit en vocabulaire de livrable |
| `docs/superpowers/specs/2026-08-26-catalogue-et-livrables-design.md` | § 6 recalé sur ce qui a été fait |

---

### Tâche 1 : La vue d'arbre — `pods_liste`

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs` (les vues, la commande, les tests)
- Modifier : `src-tauri/src/lib.rs` (le `generate_handler`)

Purement additive : rien ne l'appelle encore, l'écran ne change pas, les tests JS restent
verts. `PapierVue` gagne `dos_publie` **sans** que `ProviderVue` le perde — les deux
cohabitent le temps de deux tâches, et la tâche 4 retire le doublon en basculant ses
lecteurs. C'est le seul moment du lot où deux vérités coexistent, et il est borné.

- [x] **Étape 1 : Écrire le test qui échoue**

Dans le module de tests de `src-tauri/src/commands.rs`, à côté de
`deux_papiers_d_un_gabarit_partagent_la_mesure_sans_partager_le_dos` :

```rust
/// La vue d'arbre porte ce que la vue plate tait : les reliures d'un POD, la raison de
/// celles qu'on n'outille pas, ses finitions. C'est elle qui alimente la cascade, et
/// c'est le fichier qui tranche « composable » — `verifie_reliure` interdit qu'une
/// reliure porte à la fois une géométrie et une raison de ne pas en avoir.
#[test]
fn la_vue_d_arbre_porte_les_reliures_avec_leur_raison() {
    let pods = pods_liste();
    let bod = pods
        .iter()
        .find(|p| p.cle == "bod")
        .expect("BoD est un POD fourni");

    assert_eq!(bod.nom, "BoD");
    assert!(
        bod.formats.iter().any(|f| f.cle == "135x215"),
        "le format de BoD manque"
    );

    let broche = bod
        .reliures
        .iter()
        .find(|r| r.cle == "broche")
        .expect("BoD brochera toujours");
    assert!(
        broche.non_outille.is_none(),
        "le broché est composable : aucune raison à afficher"
    );

    let rigide = bod
        .reliures
        .iter()
        .find(|r| r.cle == "rigide")
        .expect("BoD publie une couverture rigide qu'on n'outille pas");
    let raison = rigide
        .non_outille
        .as_deref()
        .expect("une reliure non outillée dit pourquoi");
    assert!(raison.contains("casewrap"), "{raison}");
}

/// Le relevé de dos suit le **papier**, jamais le premier de la liste : un POD peut
/// publier une formule pour l'un et n'en publier aucune pour l'autre, et c'est la ligne
/// du livrable qui réclame alors la mesure.
#[test]
fn dos_publie_est_porte_par_chaque_papier() {
    let pods = pods_liste();

    let kdp = pods.iter().find(|p| p.cle == "kdp").expect("KDP est fourni");
    assert!(
        kdp.papiers.iter().all(|pa| pa.dos_publie),
        "KDP publie une formule pour ses deux papiers"
    );

    let coollibri = pods
        .iter()
        .find(|p| p.cle == "coollibri")
        .expect("CoolLibri est fourni");
    assert!(
        coollibri.papiers.iter().all(|pa| !pa.dos_publie),
        "CoolLibri ne publie aucune formule : le dos se relève sur son gabarit"
    );
}
```

- [x] **Étape 2 : Lancer le test pour le voir échouer**

Depuis `src-tauri/` :

```
cargo test la_vue_d_arbre_porte_les_reliures_avec_leur_raison
cargo test dos_publie_est_porte_par_chaque_papier
```

`cargo` n'accepte **qu'un** filtre de nom : `cargo test A B` est refusé.

Attendu : **échec de compilation** — `cannot find function 'pods_liste' in this scope`, et
`no field 'dos_publie' on type 'PapierVue'`.

- [x] **Étape 3 : Écrire les vues et la commande**

Dans `src-tauri/src/commands.rs`, à la suite de `PapierVue` (vers la ligne 82) :

```rust
/// Ce qu'un POD offre, en arbre : la cascade de l'ajout y lit ses formats, les trois
/// réglages de la ligne y lisent ses reliures, ses finitions et ses papiers.
///
/// Distincte de `ProviderVue`, et non un champ de plus sur elle : celle-là est une
/// projection POD × format, qui n'a pas de place pour dire ce qu'un POD offre d'autre.
/// Les deux cohabitent — la plate pour ce que la projection sait seule dire (format en
/// mm, fond perdu effectif, libellé composé), l'arbre pour les choix.
#[derive(Serialize)]
pub struct PodVue {
    cle: String,
    nom: String,
    formats: Vec<FormatVue>,
    reliures: Vec<ReliureVue>,
    finitions: Vec<FinitionVue>,
    papiers: Vec<PapierVue>,
}

#[derive(Serialize)]
pub struct FormatVue {
    cle: String,
    nom: String,
}

#[derive(Serialize)]
pub struct ReliureVue {
    cle: String,
    nom: String,
    /// Pourquoi on ne la compose pas, telle que le fichier l'écrit — `null` chez une
    /// reliure composable. C'est le fichier qui tranche : `verifie_reliure` refuse une
    /// reliure qui porterait à la fois une géométrie et une raison de ne pas en avoir,
    /// donc l'écran n'a pas à interroger la géométrie pour savoir quoi griser.
    non_outille: Option<String>,
}

#[derive(Serialize)]
pub struct FinitionVue {
    cle: String,
    nom: String,
}
```

`PapierVue` (`commands.rs:73-80`) gagne un champ :

```rust
#[derive(Serialize)]
pub struct PapierVue {
    cle: String,
    libelle: String,
    /// La couleur du papier, telle que le canevas des envois la peint. Elle traverse
    /// jusqu'ici parce que c'est l'écran qui s'en sert, jamais la composition.
    teinte: String,
    /// Vrai quand **ce papier** publie de quoi calculer le dos. Faux, la ligne réclame
    /// un relevé plutôt que de laisser croire à un chiffre. Porté par le papier et non
    /// par le POD : un POD peut publier une formule pour l'un et pas pour l'autre, et
    /// c'est le papier retenu qui décide.
    dos_publie: bool,
}
```

La conversion depuis un `catalogue::Papier`, à écrire une fois et à appeler des deux
côtés — `ProviderVue::from` la réutilise :

```rust
impl From<&catalogue::Papier> for PapierVue {
    fn from(pa: &catalogue::Papier) -> Self {
        Self {
            cle: pa.cle.clone(),
            libelle: pa.nom.clone(),
            teinte: pa.teinte.clone(),
            // Une pagination quelconque suffit à savoir si une formule existe.
            dos_publie: pa.dos.mm(100).is_some(),
        }
    }
}
```

Dans `ProviderVue::from` (`commands.rs:96-101`), le bloc `papiers` se réduit à :

```rust
            papiers: p.papiers.iter().map(PapierVue::from).collect(),
```

Et la commande, à la suite de `providers_liste` (`commands.rs:171-176`) :

```rust
/// L'arbre du catalogue : un POD, ses formats, ses reliures, ses finitions, ses papiers.
///
/// Ne sont rendus que les POD chez qui l'on sait composer — au moins une reliure
/// outillée —, la règle qu'`aplatit` applique déjà à la table plate. Un POD dont aucune
/// reliure n'aurait de géométrie relevée n'offre rien à ajouter, et le faire paraître
/// grisé en entier serait une fonction que personne n'a demandée.
#[tauri::command]
pub fn pods_liste() -> Vec<PodVue> {
    catalogue::pods()
        .iter()
        .filter(|pod| pod.reliure_composable().is_some())
        .map(|pod| PodVue {
            cle: pod.cle.clone(),
            nom: pod.nom.clone(),
            formats: pod
                .formats
                .iter()
                .map(|f| FormatVue {
                    cle: f.cle.clone(),
                    nom: f.nom.clone(),
                })
                .collect(),
            reliures: pod
                .reliures
                .iter()
                .map(|r| ReliureVue {
                    cle: r.cle.clone(),
                    nom: r.nom.clone(),
                    non_outille: r.non_outille.clone(),
                })
                .collect(),
            finitions: pod
                .finitions
                .iter()
                .map(|f| FinitionVue {
                    cle: f.cle.clone(),
                    nom: f.nom.clone(),
                })
                .collect(),
            papiers: pod.papiers.iter().map(PapierVue::from).collect(),
        })
        .collect()
}
```

- [x] **Étape 4 : Enregistrer la commande**

Dans `src-tauri/src/lib.rs`, à la ligne qui suit `commands::providers_liste,` (l. 96) :

```rust
            commands::pods_liste,
```

- [x] **Étape 5 : Lancer les tests pour les voir passer**

Depuis `src-tauri/` :

```
cargo test la_vue_d_arbre_porte_les_reliures_avec_leur_raison
cargo test dos_publie_est_porte_par_chaque_papier
```

Attendu : **1 passed** chacun. Puis la suite entière, `cargo test` : aucun test existant ne lit
`PapierVue`, l'ajout d'un champ ne casse rien.

- [x] **Étape 6 : Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "Le catalogue se donne en arbre, et le dos se publie par papier"
```

---

### Tâche 2 : La reliure se règle sur la ligne

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs:521-532` (`reglage_refuse`) et son test l. 2354

Rust seul : l'écran n'offre pas encore le contrôle, donc rien ne bouge à l'œil. C'est
l'assouplissement du verdict 2 — le lot 2 avait verrouillé le gabarit entier ; la spec § 6
n'en verrouille que le POD et le format.

- [x] **Étape 1 : Écrire le test qui échoue**

Dans `src-tauri/src/commands.rs`, remplacer le corps du test
`changer_le_gabarit_d_un_livrable_est_refuse_en_disant_quoi_faire` (l. 2354-2367) par
celui-ci, renommé — il dit désormais ce qui se verrouille, et non plus ce qui se ferme :

```rust
/// Le POD et le format se choisissent à l'ajout, en cascade, et ne se règlent plus : les
/// changer sur place laisserait le livrable sous une pagination qui n'est plus la
/// sienne, et le refus dit le geste qui, lui, marche. La reliure, elle, **se règle**
/// (spec § 6) : elle change le gabarit, le livrable retombe sur un gabarit sans mesure,
/// et c'est exactement ce qu'une reliure exige — sa pagination admise, sa parité et sa
/// géométrie ne sont pas celles de la précédente.
#[test]
fn le_pod_et_le_format_ne_se_reglent_pas_la_reliure_si() {
    let place = Livrable::pour(fabrication("kdp", "6x9", "broche", "creme"));

    let autre_format = Livrable::pour(fabrication("kdp", "5x8", "broche", "creme"));
    let refus = reglage_refuse(&place, &autre_format, &pod_a_finition())
        .expect("un format changé doit être refusé");
    assert!(refus.contains("retirer"), "{refus}");

    let autre_pod = Livrable::pour(fabrication("bod", "6x9", "broche", "creme"));
    let refus = reglage_refuse(&place, &autre_pod, &pod_a_finition())
        .expect("un POD changé doit être refusé");
    assert!(refus.contains("retirer"), "{refus}");

    // La reliure se règle : c'est le geste que la spec § 6 pose sur la ligne.
    let autre_reliure = Livrable::pour(fabrication("kdp", "6x9", "rigide", "creme"));
    assert_eq!(
        reglage_refuse(&place, &autre_reliure, &pod_a_finition()),
        None,
        "la reliure doit se régler sur la ligne"
    );

    // Le papier aussi, comme depuis le lot 2.
    let autre_papier = Livrable::pour(fabrication("kdp", "6x9", "broche", "blanc"));
    assert_eq!(
        reglage_refuse(&place, &autre_papier, &pod_a_finition()),
        None
    );
}
```

- [x] **Étape 2 : Lancer le test pour le voir échouer**

Depuis `src-tauri/` :

```
cargo test le_pod_et_le_format_ne_se_reglent_pas_la_reliure_si
```

Attendu : **FAIL** sur l'assertion « la reliure doit se régler sur la ligne » —
`reglage_refuse` compare encore les gabarits entiers et rend le refus « retirer, puis
ajouter ».

- [x] **Étape 3 : Assouplir le verrou**

Dans `src-tauri/src/commands.rs`, remplacer l'en-tête et le premier test de
`reglage_refuse` (l. 515-528) :

```rust
/// Ce qu'un réglage de ligne ne peut pas faire.
///
/// Le POD et le format ne se règlent pas : ils se choisissent à l'ajout, en cascade, et
/// les changer sur place laisserait le livrable sous une pagination qui n'est plus la
/// sienne — retirer puis ajouter le dit, et le fait. La reliure, elle, se règle (spec
/// § 6) : elle emporte le gabarit avec elle, le livrable retombe sur un gabarit sans
/// mesure, et la recomposition est précisément ce qu'elle exige. La finition doit
/// exister chez le POD : elle nomme une option de commande, et une option inventée ne se
/// commande nulle part.
fn reglage_refuse(place: &Livrable, neuf: &Livrable, pod: &catalogue::Pod) -> Option<String> {
    let axes = |l: &Livrable| (l.fabrication.pod.clone(), l.fabrication.format.clone());
    if axes(place) != axes(neuf) {
        return Some(
            "le POD et le format d'un livrable ne se règlent pas : retirer, puis ajouter.".into(),
        );
    }
    match &neuf.finition {
        Some(f) if !pod.finitions.iter().any(|x| &x.cle == f) => {
            Some(format!("finition inconnue chez {} : {f}.", pod.nom))
        }
        _ => None,
    }
}
```

- [x] **Étape 4 : Lancer le test pour le voir passer**

Depuis `src-tauri/` : `cargo test le_pod_et_le_format_ne_se_reglent_pas_la_reliure_si`
→ **PASS**. Puis `cargo test` entier.

- [x] **Étape 5 : Consigner la mesure orpheline**

Une mesure dont plus aucun livrable ne porte le gabarit survit en mémoire — et donc dans
le `.ozalid` réécrit — jusqu'à la prochaine ouverture, où `normalise` l'élague
(`projet.rs:441-448`, appelée seulement à l'ouverture, `projet.rs:920`). Personne ne la
lit ; ce n'est pas un défaut à corriger ici. Ajouter la note à la documentation de
`Livraison.mesures` (`src-tauri/src/projet.rs:339-341`), après la phrase existante :

```rust
    /// Une mesure dont plus aucun livrable ne porte le gabarit — une reliure réglée sur
    /// la ligne, depuis le lot 3 — survit jusqu'à la prochaine ouverture, où `normalise`
    /// l'élague. Personne ne la lit entre-temps : elle est rangée sous une clé que plus
    /// aucun livrable ne forme.
```

- [x] **Étape 6 : Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/projet.rs
git commit -m "La reliure se règle sur la ligne, le POD et le format restent au choix d'ajout"
```

---

### Tâche 3 : La cascade à l'ajout

**Fichiers :**
- Modifier : `src/index.html:303-306` (la ligne d'ajout)
- Modifier : `src/livraison.js` (le remplissage des deux listes)
- Modifier : `src/app.js:454` (chargement) et `src/app.js:1289-1297` (le bouton)
- Modifier : les neuf faux `invoke` — `tests/{composition,contrats,coquille,couverture,cycle_de_vie,dom_shim,ebook,epreuve,packages}.test.js`
- Modifier : `tests/packages.test.js` (les tests de l'ajout)

C'est la tâche qui apprend `pods_liste` à tous les faux : sans elle, neuf fichiers de
tests jettent « commande inattendue ».

- [x] **Étape 1 : Écrire le test qui échoue**

Dans `tests/packages.test.js`, la constante d'arbre à poser près de `LULU`/`KDP`/`COOLLIBRI` :

```js
// L'arbre du catalogue, tel que `pods_liste` le rend. Volontairement plus riche que la
// table plate des tests : c'est lui qui porte les choix, et le grisé motivé n'a rien à
// lire ailleurs.
const PODS = [
  {
    cle: 'lulu', nom: 'Lulu',
    formats: [{ cle: '108x175', nom: 'poche 108 × 175' }],
    reliures: [{ cle: 'broche', nom: 'Broché — dos carré collé', non_outille: null }],
    finitions: [],
    papiers: [{ cle: 'standard', libelle: 'Papier standard', teinte: '#ffffff', dos_publie: true }],
  },
  {
    cle: 'kdp', nom: 'Amazon KDP',
    formats: [{ cle: '6x9', nom: '6 × 9 po' }, { cle: '5x8', nom: '5 × 8 po' }],
    reliures: [
      { cle: 'broche', nom: 'Broché — dos carré collé', non_outille: null },
      { cle: 'rigide', nom: 'Couverture rigide', non_outille: 'géométrie du casewrap non relevée' },
    ],
    finitions: [{ cle: 'mat', nom: 'Pelliculage mat' }],
    papiers: [
      { cle: 'creme', libelle: 'Crème', teinte: '#f7f0e0', dos_publie: true },
      { cle: 'blanc', libelle: 'Blanc', teinte: '#ffffff', dos_publie: true },
    ],
  },
  {
    cle: 'coollibri', nom: 'CoolLibri',
    formats: [{ cle: '148x210', nom: 'A5' }],
    reliures: [{ cle: 'broche', nom: 'Broché — dos carré collé', non_outille: null }],
    finitions: [],
    papiers: [{ cle: 'mesure', libelle: 'Dos relevé sur le gabarit', teinte: '#ffffff', dos_publie: false }],
  },
];
```

Et les deux tests, à la suite de ceux qui exercent l'ajout :

```js
test('la cascade offre les formats du POD choisi, et eux seuls', async () => {
  const { els } = await ouvre([LULU, KDP, COOLLIBRI], {}, { pods: PODS });

  assert.deepStrictEqual(
    els.get('inAjoutPod').textes('option'),
    ['Lulu', 'Amazon KDP', 'CoolLibri'],
    'la liste des POD ne les donne pas tous, ou pas dans l\'ordre du catalogue'
  );
  // Le premier POD est choisi d'office : une cascade qui commence vide demande un clic
  // pour ne rien dire.
  assert.deepStrictEqual(els.get('inAjoutFormat').textes('option'), ['poche 108 × 175']);

  els.get('inAjoutPod').value = 'kdp';
  await els.get('inAjoutPod').declenche('change');
  assert.deepStrictEqual(
    els.get('inAjoutFormat').textes('option'),
    ['6 × 9 po', '5 × 8 po'],
    'changer de POD n\'a pas rechargé ses formats'
  );
});

test('l\'ajout envoie les quatre axes, la reliure composable et le premier papier', async () => {
  const { els, appels } = await ouvre([LULU, KDP, COOLLIBRI], {}, { pods: PODS });

  els.get('inAjoutPod').value = 'kdp';
  await els.get('inAjoutPod').declenche('change');
  els.get('inAjoutFormat').value = '5x8';
  await els.get('btAjouterDestinataire').declenche('click');

  const [, args] = appels.findLast(([cmd]) => cmd === 'livrable_ajouter');
  assert.deepStrictEqual(args.fabrication, {
    // La reliure d'office est la première **composable** : la rigide de KDP porte une
    // raison de ne pas l'être, et le Rust la refuserait.
    pod: 'kdp', format: '5x8', reliure: 'broche', papier: 'creme',
  });
});
```

Le faux `ouvre` de `packages.test.js` prend une option de plus (l. 115-118) :

```js
async function ouvre(
  providers,
  sur = {},
  { couverture = null, destinataires, dejaCompose = false, dosParPapier = {}, pods = [] } = {}
) {
```

et sert la commande, à côté de `providers_liste` (l. 162) :

```js
    if (cmd === 'pods_liste') return pods;
```

- [x] **Étape 2 : Lancer les tests pour les voir échouer**

Depuis la racine :

```
node --test tests/packages.test.js
```

Attendu : **FAIL** — `els.get('inAjoutPod')` est `undefined` : l'identifiant n'existe pas
dans `index.html`, et le shim ne fabrique que ce qu'il y lit.

- [x] **Étape 3 : Poser les deux listes dans le balisage**

Dans `src/index.html`, remplacer la ligne d'ajout (l. 303-306) :

```html
      <div class="ligne">
        <!-- Deux listes en cascade, et le seul endroit de la fenêtre où des contrôles
             n'ont pas d'étiquette visible : posés contre leur bouton, ils se lisent
             comme le geste d'ajout. Ce que l'œil déduit de la disposition, les
             `aria-label` le disent à qui ne la voit pas.
             Le POD d'abord, ses formats ensuite : un format ne veut rien dire sans
             l'imprimeur qui le fabrique, et les mêmes 13,5 × 21,5 n'ont pas les mêmes
             marges chez deux POD. -->
        <select id="inAjoutPod" aria-label="Imprimeur à ajouter"></select>
        <select id="inAjoutFormat" aria-label="Format du livrable à ajouter"></select>
        <button id="btAjouterDestinataire" type="button">Ajouter</button>
      </div>
```

- [x] **Étape 4 : Charger l'arbre au démarrage**

Dans `src/app.js`, à côté de `providers` (l. 20) :

```js
let pods = [];
```

et dans `chargerProviders` (l. 454) :

```js
  providers = await invoke('providers_liste');
  // L'arbre du catalogue : ce que chaque POD offre. La table plate ci-dessus reste la
  // seule à savoir dire un format en millimètres et un fond perdu effectif ; celle-ci
  // est la seule à savoir dire ce qu'on a le droit de choisir.
  pods = await invoke('pods_liste');
```

- [x] **Étape 5 : Remplir la cascade**

Dans `src/livraison.js`, remplacer les six dernières lignes d'`afficherDestinataires`
(l. 114-121, du commentaire « La table entière… » à la ligne
`$('btAjouterDestinataire').disabled = providers.length === 0;`) par un appel :

```js
  afficherCascade();
}

/**
 * Les deux listes de l'ajout : le POD, puis **ses** formats.
 *
 * Aucun filtre sur ce qui est déjà déclaré : c'est ce qui permet de déclarer deux fois
 * le même gabarit pour comparer deux papiers. Le vrai doublon — les quatre axes
 * identiques — est refusé par le Rust, avec sa raison.
 *
 * La liste des POD se reconstruit à chaque affichage, celle des formats la suit : elles
 * ne dépendent que du catalogue, qui ne bouge pas de la vie du processus, mais les
 * reconstruire coûte deux boucles sur six entrées et évite d'avoir à se demander qui les
 * a laissées dans quel état.
 */
function afficherCascade() {
  const sel = $('inAjoutPod');
  const choisi = sel.value;
  sel.replaceChildren();
  for (const p of pods) sel.append(new Option(p.nom, p.cle));
  // Le POD retenu survit à un réaffichage : ajouter un livrable ne doit pas ramener la
  // liste sur son premier, alors qu'on en ajoute souvent deux de suite chez le même.
  if (pods.some((p) => p.cle === choisi)) sel.value = choisi;
  sel.disabled = pods.length === 0;
  $('btAjouterDestinataire').disabled = pods.length === 0;
  afficherFormatsDuPod();
}

/** Les formats du POD choisi. Vidée et refaite : un format d'un autre POD ne veut rien dire. */
function afficherFormatsDuPod() {
  const p = pods.find((x) => x.cle === $('inAjoutPod').value);
  const sel = $('inAjoutFormat');
  sel.replaceChildren();
  for (const f of p?.formats ?? []) sel.append(new Option(f.nom, f.cle));
  sel.disabled = !p || p.formats.length < 2;
}
```

- [x] **Étape 6 : Brancher les deux écouteurs**

Dans `src/app.js`, remplacer l'écouteur du bouton d'ajout (l. 1289-1297) :

```js
// La cascade parle en POD puis en format ; la reliure et le papier d'office viennent du
// catalogue, et se règlent ensuite sur la ligne. C'est le Rust qui refuse le vrai
// doublon.
$('inAjoutPod').addEventListener('change', afficherFormatsDuPod);
$('btAjouterDestinataire').addEventListener('click', () => tente(async () => {
  const p = pods.find((x) => x.cle === $('inAjoutPod').value);
  // La première reliure **composable** : une reliure grisée porte une raison de ne pas
  // l'être, et le Rust la refuserait en la citant. Proposer d'office ce qu'on sait
  // refuser serait un piège tendu au premier clic.
  const reliure = p.reliures.find((r) => r.non_outille === null);
  afficherProjet(await invoke('livrable_ajouter', {
    fabrication: {
      pod: p.cle,
      format: $('inAjoutFormat').value,
      reliure: reliure.cle,
      papier: p.papiers[0].cle,
    },
  }));
}));
```

- [x] **Étape 7 : Apprendre `pods_liste` aux huit autres faux**

Dans chacun de `tests/composition.test.js`, `tests/contrats.test.js`,
`tests/coquille.test.js`, `tests/couverture.test.js`, `tests/cycle_de_vie.test.js`,
`tests/dom_shim.test.js`, `tests/ebook.test.js`, `tests/epreuve.test.js` : à la ligne qui
suit le `providers_liste` du faux `invoke`, servir un arbre cohérent avec la table plate
que ce fichier utilise déjà. Pour un fichier qui ne sert que `LULU` (`contrats`,
`cycle_de_vie`, `couverture`) :

```js
  if (cmd === 'pods_liste') return [{
    cle: 'lulu', nom: 'Lulu',
    formats: [{ cle: '108x175', nom: 'poche 108 × 175' }],
    reliures: [{ cle: 'broche', nom: 'Broché — dos carré collé', non_outille: null }],
    finitions: [],
    papiers: [{ cle: 'standard', libelle: 'Papier standard', teinte: '#ffffff', dos_publie: true }],
  }];
```

Les autres, nommément :

- `tests/composition.test.js` déclare `LULU`, `KDP` et `COOLLIBRI` (l. 9, 15, 21) : lui
  donner la constante `PODS` de l'étape 1, telle quelle.
- `tests/ebook.test.js` (l. 11) et `tests/epreuve.test.js` (l. 11) ne déclarent que
  `LULU` : l'arbre à un seul POD ci-dessus suffit.
- `tests/coquille.test.js` sert une liste variable (l. 107) : lui donner `PODS` aussi,
  ses tests portant sur `KDP` crème/blanc.
- `tests/dom_shim.test.js` (l. 39) n'a besoin que du POD qu'il déclare déjà, avec les
  quatre listes vides sauf `formats`, `reliures` et `papiers` à une entrée.

Chaque arbre doit rester **cohérent avec la table plate du même fichier** : une clé de POD
qui ne s'y retrouve pas ferait rendre `undefined` à la cascade, et l'ajout échouerait sur
un `p.reliures` illisible plutôt que sur une assertion.

**Le test de forme, à ajouter dans `tests/contrats.test.js`** — c'est le fichier des
contrats, et un faux muet sur une commande du démarrage est exactement ce qu'il garde :

```js
test('le démarrage ne demande que des commandes que le Rust expose', async () => {
  const lib = fs.readFileSync(path.join(__dirname, '..', 'src-tauri', 'src', 'lib.rs'), 'utf8');
  for (const cmd of ['providers_liste', 'pods_liste', 'catalogue_refus']) {
    assert.ok(
      lib.includes(`commands::${cmd},`),
      `${cmd} n'est pas enregistrée dans le generate_handler`
    );
  }
});
```

- [x] **Étape 8 : Lancer la suite pour la voir passer**

Depuis la racine : `node --test tests/*.test.js`.

Attendu : **0 échec**, et la suite **rend la main**. Si elle boucle au-delà d'une minute,
c'est qu'un faux a été changé sans ses lecteurs (piège transverse).

- [x] **Étape 9 : Commit**

```bash
git add src/index.html src/app.js src/livraison.js tests/
git commit -m "L'ajout se fait en cascade : l'imprimeur, puis ses formats"
```

---

### Tâche 4 : Les trois réglages sur la ligne, et le grisé qui dit pourquoi

**Fichiers :**
- Modifier : `src/livraison.js:67-133` (`afficherDestinataires`, `champReleve`, `reglerLivrable`)
- Modifier : `src/app.js:386` (le pied) et `src/app.js:491-501` (un helper de papier)
- Modifier : `src-tauri/src/commands.rs` (retrait de `ProviderVue.dos_publie`)
- Modifier : `tests/packages.test.js`, `tests/composition.test.js` (les fixtures et les tests)

Le cœur du lot. La ligne passe d'un contrôle à trois, et `dos_publie` bascule du POD au
papier — les deux vérités de la tâche 1 se réduisent à une.

- [x] **Étape 1 : Écrire les tests qui échouent**

Dans `tests/packages.test.js` :

```js
test('la ligne offre les reliures du POD, la non outillée grisée avec sa raison', async () => {
  const { els } = await ouvre([KDP], {}, { pods: PODS, destinataires: [chez(KDP)] });

  const reliures = els.get('dest-reliure-kdp-6x9-broche-creme');
  assert.deepStrictEqual(
    reliures.textes('option'),
    ['Broché — dos carré collé', 'Couverture rigide']
  );

  const [broche, rigide] = reliures.children;
  assert.strictEqual(broche.disabled, false);
  assert.strictEqual(rigide.disabled, true, 'une reliure non outillée doit être grisée');

  // La raison en clair, sous la ligne — pas dans une infobulle : c'est la différence
  // entre « ce POD ne le fait pas » et « l'application ne le compose pas », et elle
  // doit se lire à l'écran.
  const raison = els.get('dest-reliure-raison-kdp-6x9-broche-creme');
  assert.match(raison.textContent, /casewrap/);
});

// Deux reliures composables chez le même POD : le cas que la reliure réglable rend
// possible, et qu'aucun POD fourni n'offre encore — BoD a bien deux reliures, mais l'une
// n'est pas outillée, et un livrable ne peut pas vivre dessus. Le catalogue le permet
// depuis le lot 1 : `pages` et `parite` vivent sur la reliure, précisément parce que deux
// reliures d'un même format n'admettent pas la même pagination.
const DEUX_RELIURES = {
  cle: 'tbe', nom: 'TheBookEdition',
  formats: [{ cle: '148x210', nom: 'A5' }],
  reliures: [
    { cle: 'broche', nom: 'Broché — dos carré collé', non_outille: null },
    { cle: 'spirale', nom: 'Reliure spirale', non_outille: null },
  ],
  finitions: [],
  papiers: [{ cle: 'munken-80', libelle: 'Munken 80 g', teinte: '#f7f0e0', dos_publie: true }],
};
const TBE_BROCHE = {
  cle: 'tbe-148x210-broche', pod: 'tbe', format: '148x210', reliure: 'broche',
  libelle: 'TheBookEdition — A5', largeur: 148, hauteur: 210, fond_perdu: 3,
  papiers: DEUX_RELIURES.papiers,
};
const TBE_SPIRALE = { ...TBE_BROCHE, cle: 'tbe-148x210-spirale', reliure: 'spirale' };

test('régler la reliure renvoie les quatre axes au Rust', async () => {
  const { els, appels } = await ouvre(
    [TBE_BROCHE, TBE_SPIRALE],
    {},
    { pods: [DEUX_RELIURES], destinataires: [chez(TBE_BROCHE)] }
  );

  const reliures = els.get('dest-reliure-tbe-148x210-broche-munken-80');
  reliures.value = 'spirale';
  await reliures.declenche('change');

  const [, args] = appels.findLast(([cmd]) => cmd === 'livrable_regler');
  assert.strictEqual(args.cle, 'tbe-148x210-broche-munken-80');
  assert.strictEqual(args.livrable.reliure, 'spirale');
  assert.strictEqual(args.livrable.pod, 'tbe');
  assert.strictEqual(args.livrable.format, '148x210');
});

test('la finition ne paraît que chez un POD qui en déclare', async () => {
  const chezKdp = await ouvre([KDP], {}, { pods: PODS, destinataires: [chez(KDP)] });
  const finitions = chezKdp.els.get('dest-finition-kdp-6x9-broche-creme');
  assert.ok(finitions, 'KDP déclare une finition : le contrôle doit être là');
  // Le vide en tête : aucune finition est le cas courant, et il doit rester choisissable.
  assert.deepStrictEqual(finitions.textes('option'), ['—', 'Pelliculage mat']);

  const chezLulu = await ouvre([LULU], {}, { pods: PODS });
  assert.ok(
    !chezLulu.els.get('dest-finition-lulu-108x175-broche-standard'),
    'un POD sans finition ne doit pas offrir un contrôle vide'
  );
});

test('le relevé de dos suit le papier, pas le POD', async () => {
  // Un POD dont un papier publie sa formule et l'autre pas : le cas que la table plate
  // ne savait pas dire, puisqu'elle tranchait sur le papier d'office.
  const mixte = {
    cle: 'mixte', nom: 'Mixte',
    formats: [{ cle: 'a5', nom: 'A5' }],
    reliures: [{ cle: 'broche', nom: 'Broché', non_outille: null }],
    finitions: [],
    papiers: [
      { cle: 'formule', libelle: 'Papier à formule', teinte: '#ffffff', dos_publie: true },
      { cle: 'gabarit', libelle: 'Papier à relever', teinte: '#ffffff', dos_publie: false },
    ],
  };
  const plat = {
    cle: 'mixte-a5-broche', pod: 'mixte', format: 'a5', reliure: 'broche',
    libelle: 'Mixte — A5', largeur: 148, hauteur: 210, fond_perdu: 3,
    papiers: mixte.papiers,
  };
  const { els } = await ouvre([plat], {}, { pods: [mixte] });

  assert.ok(
    !els.get('dest-dos-mixte-a5-broche-formule'),
    'le papier à formule ne doit pas réclamer de relevé'
  );

  els.get('dest-papier-mixte-a5-broche-formule').value = 'gabarit';
  await els.get('dest-papier-mixte-a5-broche-formule').declenche('change');
  assert.ok(
    els.get('dest-dos-mixte-a5-broche-gabarit'),
    'le papier à relever doit réclamer son dos'
  );
});
```

- [x] **Étape 2 : Lancer les tests pour les voir échouer**

Depuis la racine : `node --test tests/packages.test.js`.

Attendu : **FAIL** — `dest-reliure-*`, `dest-finition-*` et `dest-reliure-raison-*`
n'existent pas ; le dernier test échoue sur `dest-dos-mixte-a5-broche-formule`, que la
ligne pose encore d'après `p.dos_publie` du provider (absent de la fixture, donc
`undefined`, donc « faux »).

- [x] **Étape 3 : Refaire la ligne**

Dans `src/livraison.js`, remplacer le corps de la boucle d'`afficherDestinataires`
(l. 71-112, du `for (const d of declares)` au `}` qui la ferme) :

```js
  for (const d of declares) {
    const p = providers.find((pr) => pr.cle === d.gabarit);
    const pod = pods.find((x) => x.cle === d.pod);
    const ligne = h('div', undefined, 'destinataire');
    let releve;
    let raison;
    ligne.append(h('span', libelleProvider(d.gabarit), 'nom'));

    if (pod) {
      // Les reliures du POD, la non outillée grisée : le Rust la refuse déjà en citant
      // sa raison (`catalogue::resout`), et l'écran ne fait que rendre ce refus lisible
      // avant le clic. Le fichier tranche — une reliure porte une géométrie **ou** une
      // raison de ne pas en avoir, jamais les deux.
      const reliure = h('select');
      reliure.id = `dest-reliure-${d.cle}`;
      for (const r of pod.reliures) {
        const o = new Option(r.nom, r.cle);
        o.disabled = r.non_outille !== null;
        reliure.append(o);
      }
      reliure.value = d.reliure;
      // Éteint seulement quand le POD n'a **qu'une** reliure, toutes confondues : un
      // select éteint ne s'ouvre pas, et l'éteindre dès qu'il n'y a qu'une composable
      // cacherait justement le grisé que la spec § 6 demande de montrer — c'est le cas
      // de BoD, le seul POD fourni qui en porte un.
      reliure.disabled = pod.reliures.length < 2;
      reliure.addEventListener('change', () => reglerLivrable(d));
      ligne.append(reliure);

      // La raison, en clair et sur sa propre ligne. Pas une infobulle : c'est la seule
      // partie du message qui distingue « ce POD ne le fait pas » de « l'application ne
      // le compose pas », et elle doit se lire sans survol.
      const grisees = pod.reliures.filter((r) => r.non_outille !== null);
      if (grisees.length) {
        raison = h('p', undefined, 'note raison');
        raison.id = `dest-reliure-raison-${d.cle}`;
        raison.textContent = grisees
          .map((r) => `${r.nom} — ${r.non_outille}`)
          .join(' · ');
      }

      // La finition ne paraît que là où il y en a : un contrôle vide se lit comme un
      // choix qu'on n'a pas su faire, alors qu'il n'y en avait aucun à faire. Aucun POD
      // fourni n'en déclare aujourd'hui ; c'est le lot 4 qui les relèvera.
      if (pod.finitions.length) {
        const finition = h('select');
        finition.id = `dest-finition-${d.cle}`;
        // Le vide en tête : aucune finition est le cas courant, et il doit rester
        // choisissable après en avoir pris une.
        finition.append(new Option('—', ''));
        for (const f of pod.finitions) finition.append(new Option(f.nom, f.cle));
        finition.value = d.finition ?? '';
        finition.addEventListener('change', () => reglerLivrable(d));
        ligne.append(finition);
      }

      const papier = h('select');
      papier.id = `dest-papier-${d.cle}`;
      for (const pa of pod.papiers) papier.append(new Option(pa.libelle, pa.cle));
      papier.value = d.papier;
      papier.disabled = pod.papiers.length < 2;
      papier.addEventListener('change', () => reglerLivrable(d));
      ligne.append(papier);

      // Fabriqué ici, avec le POD qui le motive, mais posé après le bouton : le relevé
      // prend une ligne à lui, et l'insérer avant renverrait le format et le bouton
      // « Retirer » au rang suivant, décalés de ceux des voisins. Ordre du balisage et
      // ordre de lecture restent les mêmes — c'est le CSS qui met le relevé à la ligne.
      // Le dos se réclame d'après **le papier retenu**, jamais d'après le POD : un POD
      // peut publier une formule pour l'un de ses papiers et pas pour l'autre.
      const dosPublie = pod.papiers.find((pa) => pa.cle === d.papier)?.dos_publie ?? false;
      if (!dosPublie || p?.fond_perdu === null) {
        releve = h('span', undefined, 'releve');
        const champ = (quoi, libelle, valeur) =>
          releve.append(champReleve(`dest-${quoi}-${d.cle}`, libelle, valeur, d));
        if (!dosPublie) champ('dos', 'Dos relevé (mm)', d.dos_mm);
        if (p?.fond_perdu === null) champ('fp', 'Fond perdu (mm)', d.fond_perdu_mm);
      }
      if (p) ligne.append(h('span', noteFormat(p), 'note'));
    }

    const retirer = h('button', 'Retirer');
    retirer.type = 'button';
    retirer.id = `dest-retirer-${d.cle}`;
    // Le dernier ne se retire pas : le Rust refuse, mais un bouton qui ne peut
    // qu'échouer vaut mieux éteint que refusé.
    retirer.disabled = declares.length < 2;
    retirer.addEventListener('click', () => tente(async () =>
      afficherProjet(await invoke('livrable_retirer', { cle: d.cle }))));
    ligne.append(retirer);
    if (releve) ligne.append(releve);
    if (raison) ligne.append(raison);
    box.append(ligne);
  }
```

- [x] **Étape 4 : Renvoyer les trois réglages au Rust**

Dans `src/livraison.js`, le corps de `reglerLivrable` (l. 156-172) — la reliure et la
finition rejoignent le papier :

```js
async function reglerLivrable(d) {
  // Un champ vide est une absence de relevé, pas un zéro : composer sur un dos nul
  // produirait une planche fausse au lieu d'un refus.
  const lu = (id) => {
    const v = $(id)?.value.trim();
    return v ? Number(v) : null;
  };
  // Un contrôle absent laisse la valeur qu'il portait : la finition n'a pas de contrôle
  // chez un POD qui n'en déclare aucune, et la ligne ne doit pas l'effacer pour autant.
  const choix = (id, defaut) => $(id)?.value ?? defaut;
  await tente(async () => afficherProjet(await invoke('livrable_regler', {
    cle: d.cle,
    livrable: {
      pod: d.pod,
      format: d.format,
      reliure: choix(`dest-reliure-${d.cle}`, d.reliure),
      papier: choix(`dest-papier-${d.cle}`, d.papier),
      // La chaîne vide du choix « — » est une absence, pas une finition nommée.
      finition: choix(`dest-finition-${d.cle}`, d.finition ?? '') || null,
      dos_mm: lu(`dest-dos-${d.cle}`),
      fond_perdu_mm: lu(`dest-fp-${d.cle}`),
    },
  })));
}
```

- [x] **Étape 5 : Basculer le pied sur le papier**

Dans `src/app.js`, ajouter le helper à côté de `providerCourant` (après la l. 492) :

```js
/**
 * Le papier du livrable visé, tel que le catalogue le décrit.
 *
 * L'arbre et non la table plate : c'est le papier retenu qui dit si le dos se calcule,
 * et la projection ne connaît que celui d'office de son POD.
 */
function papierCourant() {
  const d = livrableCourant();
  return pods.find((p) => p.cle === d?.pod)?.papiers.find((pa) => pa.cle === d?.papier);
}
```

et remplacer la ligne 386 :

```js
    : !papierCourant()?.dos_publie ? 'dos relevé sur le gabarit'
```

- [x] **Étape 6 : Retirer la seconde vérité, côté Rust**

Dans `src-tauri/src/commands.rs`, supprimer le champ `dos_publie` de `ProviderVue`
(l. 67-69) et la ligne qui le calculait dans `ProviderVue::from` (l. 93-94). Plus aucun
lecteur ne le lit : le front est passé au papier à l'étape 5.

Retirer aussi `dos_publie` des fixtures de providers dans `tests/composition.test.js`,
`tests/contrats.test.js`, `tests/couverture.test.js`, `tests/cycle_de_vie.test.js`,
`tests/ebook.test.js`, `tests/epreuve.test.js`, `tests/packages.test.js` — le porter là
ferait croire qu'il est encore servi.

- [x] **Étape 7 : Lancer les deux suites pour les voir passer**

Depuis la racine : `node --test tests/*.test.js` → **0 échec**.
Depuis `src-tauri/` : `cargo test` → **0 échec**, puis `cargo run --example temoin` →
**98 pages, dos 7,21 mm**.

- [x] **Étape 8 : Commit**

```bash
git add src/livraison.js src/app.js src-tauri/src/commands.rs tests/
git commit -m "La ligne se règle en trois axes, et la reliure grisée dit pourquoi"
```

---

### Tâche 4bis : La table plate porte toutes les reliures composables

*Tâche insérée après coup, le 26/08, sur une trouvaille de la revue de la tâche 4 —
arbitrée par l'utilisateur : on corrige la cause.*

**Fichiers :**
- Modifier : `src-tauri/src/catalogue.rs` — `aplatit` et son test
- Modifier : `tests/packages.test.js` — le commentaire des deux constantes `TBE_*`

**Le défaut.** `aplatit` n'émet qu'une entrée par couple POD × format, construite sur la
**première** reliure composable (via `pod.fabrication_defaut()`). Tant que la reliure d'un
livrable était figée, aucun livrable ne pouvait désigner un gabarit absent de cette table.
La tâche 2 a rendu la reliure réglable : chez un POD à **deux** reliures composables,
régler la reliure produit un `cle_gabarit` que la table plate ne contient pas, et le front
dégrade **en silence** — ligne intitulée par sa clé brute, note de format disparue, champ
« Fond perdu » jamais proposé (le test `p?.fond_perdu === null` est faux quand `p` est
absent), pied et sélecteur « Vu pour » escamotés.

**Ce qui le rendait invisible.** Aucun POD fourni n'a deux reliures composables — BoD en a
deux, dont une non outillée sur laquelle aucun livrable ne peut vivre. Le cas demande un
`<config>/pods/*.toml` déposé. Et le test qui aurait dû l'attraper l'endormait : il servait
une table plate à deux entrées pour un même POD × format, que `aplatit` ne savait pas
produire.

**La correction.** Une entrée par POD × format × reliure composable, chacune portant la
pagination de **sa** reliure — ce que la spec dit déjà : la pagination vit sur la reliure
« précisément parce que TheBookEdition accepte 40 à 750 pages en dos carré collé et 24 à
300 en rigide, au même format ». Les reliures bouclent **à l'extérieur** des formats, pour
que la première entrée reste (première reliure composable, premier format) et que
l'invariant de `Pod::fabrication_defaut` tienne — un livre neuf et la première ligne de la
table doivent désigner le même livrable.

Le `libelle` ne gagne **pas** la reliure : elle se lit dans son propre contrôle sur la
ligne (décision 5). C'est le libellé du **livrable** qui la porte, à la tâche 6, parce
qu'il sert le pied et les comptes rendus de package, où aucun contrôle ne se lit.

**Ce qui ne doit pas bouger.** Aucun POD livré n'ayant deux reliures composables, la table
plate est identique sur le catalogue fourni : les tests d'ancrage des quatorze livrables,
le témoin (**98 pages, dos 7,21 mm**) et la suite JS doivent passer sans modification. Un
écart signalerait que la correction a débordé.

---

### Tâche 5 : La disposition d'une ligne à quatre contrôles

**Fichiers :**
- Modifier : `src/styles.css:1095-1141`

Sans test : une disposition se vérifie à l'œil. La ligne était calibrée au pixel pour
trois éléments (verdict 5) ; elle en porte cinq.

- [x] **Étape 1 : Reprendre la ligne**

Dans `src/styles.css`, après `.destinataire select { width: auto; }` (l. 1118) :

```css
/* Trois réglages au lieu d'un : la ligne ne tient plus ses contrôles et sa note sur un
   seul rang à 1040 px. Le nom garde sa base, les trois sélecteurs se suivent sans se
   comprimer — un `select` réduit à sa flèche ne se lit plus —, et c'est la note qui
   passe au rang suivant, comme le relevé le fait déjà. */
.destinataire select { flex: 0 0 auto; max-width: 14rem; }

/* La raison d'une reliure grisée : sous la ligne, en clair, sur toute la largeur. Elle
   commente un contrôle et non un livrable — même famille que le relevé, donc même
   `flex-basis`, et le gris de la note plutôt que le rouge de l'alerte : rien n'est en
   panne, une option n'est simplement pas outillée. */
.destinataire .raison { flex-basis: 100%; margin: 0; }
```

Et la note de format (l. 1127) reprend sa base pour laisser passer trois sélecteurs :

```css
.destinataire .note { margin: 0; flex: 1 1 100%; min-width: 0; text-align: right; }
```

- [x] **Étape 2 : Regarder**

```
touch src-tauri/src/lib.rs
cd src-tauri && cargo run
```

Sur un livre à deux livrables, dont un chez KDP (deux papiers) : la fenêtre à
**1040 × 780**, la taille de `tauri.conf.json` et celle où tous les calibrages précédents
ont été mesurés (`src/styles.css:307`). À vérifier :
- les trois sélecteurs se lisent en entier, aucun réduit à sa flèche ;
- le bouton « Retirer » finit sa ligne au même endroit d'un livrable à l'autre ;
- aucun ascenseur **horizontal** ;
- chez BoD, la ligne de raison paraît sous la ligne, en gris, lisible sans survol.

- [x] **Étape 3 : Commit**

```bash
git add src/styles.css
git commit -m "La ligne de livraison range quatre contrôles et la raison du grisé"
```

---

### Tâche 6 : Le libellé porte la reliure

**Fichiers :**
- Modifier : `src/app.js:505-513` (`libelleLivrable`)
- Modifier : `tests/packages.test.js`

`libelleLivrable` sert le pointeur du pied et les comptes rendus de package, où aucun
contrôle ne se lit. Avec la reliure réglable, deux livrables peuvent ne différer que par
elle et s'y lire identiques (décision 6).

- [x] **Étape 1 : Écrire le test qui échoue**

Dans `tests/packages.test.js` :

Les fixtures `DEUX_RELIURES`, `TBE_BROCHE` et `TBE_SPIRALE` sont celles de la tâche 4 :
un livrable ne peut pas vivre sur une reliure non outillée — `catalogue::resout` le
refuse, et `normalise` l'élaguerait à l'ouverture —, donc le test a besoin d'un POD à
**deux reliures composables**.

```js
test('deux livrables du même papier se distinguent par leur reliure au pied', async () => {
  const broche = chez(TBE_BROCHE);
  const spirale = {
    ...chez(TBE_SPIRALE),
    cle: 'tbe-148x210-spirale-munken-80',
    gabarit: 'tbe-148x210-spirale',
  };
  const { els } = await ouvre(
    [TBE_BROCHE, TBE_SPIRALE],
    {},
    { pods: [DEUX_RELIURES], destinataires: [broche, spirale] }
  );

  const [un, deux] = els.get('inDestinataire').textes('option');
  assert.notStrictEqual(un, deux, 'deux livrables ne doivent jamais se lire identiques');
  assert.match(un, /Broché/);
  assert.match(deux, /spirale/i);
});
```

- [x] **Étape 2 : Lancer le test pour le voir échouer**

`node --test tests/packages.test.js` → **FAIL** sur `notStrictEqual` : les deux options
portent « Amazon KDP — 6 × 9 po — Crème ».

- [x] **Étape 3 : Ajouter la reliure au libellé**

Dans `src/app.js`, remplacer `libelleLivrable` (l. 505-513) :

```js
/**
 * Le libellé d'un livrable : son gabarit, sa reliure, et le papier qui le distingue.
 *
 * Ni la reliure ni le papier ne sont des ornements ici : deux livrables du même POD et
 * du même format ne se distinguent que par eux, et le pied — comme le compte rendu d'un
 * package — les donnerait à lire identiques sans eux. Sur la ligne de la Livraison, en
 * revanche, les deux se lisent dans leurs contrôles : le libellé n'a rien à y ajouter.
 */
function libelleLivrable(d) {
  const p = providers.find((x) => x.cle === d.gabarit);
  const pod = pods.find((x) => x.cle === d.pod);
  const reliure = pod?.reliures.find((x) => x.cle === d.reliure)?.nom ?? d.reliure;
  const papier = pod?.papiers.find((x) => x.cle === d.papier)?.libelle ?? d.papier;
  return `${p?.libelle ?? d.gabarit} — ${reliure} — ${papier}`;
}
```

- [x] **Étape 4 : Lancer les tests pour les voir passer**

`node --test tests/*.test.js` → **0 échec**. Les tests qui ancraient l'ancien libellé
(`coquille.test.js`, `packages.test.js`) sont à recaler sur la forme à trois segments —
c'est le même libellé, avec sa reliure.

- [x] **Étape 5 : Commit**

```bash
git add src/app.js tests/
git commit -m "Un livrable se nomme par sa reliure autant que par son papier"
```

---

### Tâche 7 : Le vocabulaire passe au livrable

**Fichiers :**
- Modifier : `src/index.html`, `src/app.js`, `src/livraison.js`, `src/couverture.js`, `src/envois.js`, `src/styles.css`
- Modifier : `tests/{packages,coquille,composition,couverture}.test.js`
- Modifier : `README.md` (§ « Le prestataire, choisi une seule fois »)

Purement mécanique, et **séparé exprès** : un renommage de ~160 points noyé dans la refonte
de l'écran rend la revue de la cascade illisible (décision 4). Aucun test neuf — la suite
existante est le filet, et elle doit rester verte à l'identique.

Le tableau des renommages, exhaustif :

| avant | après | où |
|---|---|---|
| `afficherDestinataires` | `afficherLivrables` | `livraison.js`, `app.js` |
| `#destinataires` | `#livrables` | `index.html`, `livraison.js`, `styles.css` |
| `.destinataire` / `.destinataires` | `.livrable` / `.livrables` | `styles.css`, `livraison.js` |
| `dest-papier-`, `dest-reliure-`, `dest-finition-`, `dest-dos-`, `dest-fp-`, `dest-retirer-`, `dest-reliure-raison-` | `liv-…` | `livraison.js`, tests |
| `#inDestinataire` | `#inLivrable` | `index.html`, `app.js`, tests |
| `#btAjouterDestinataire` | `#btAjouterLivrable` | `index.html`, `app.js`, `livraison.js`, tests |
| `<h2>Destinataires</h2>` | `<h2>Livrables</h2>` | `index.html` |

- [x] **Étape 1 : Relever le point de départ**

Depuis la racine, pour pouvoir comparer :

```
node --test tests/*.test.js 2>&1 | tail -5
grep -rc "estinataire" src/*.js src/*.html src/*.css
```

Attendu : **0 échec**, et 60 occurrences réparties comme au verdict 6 de la reconnaissance.

- [x] **Étape 2 : Renommer dans `src/`**

Les identifiants et les symboles du tableau ci-dessus, un fichier à la fois. Le mot
« destinataire » subsiste **dans les commentaires et les textes d'interface** partout où il
désigne encore la personne ou l'imprimeur à qui l'on livre — ce n'est pas le même mot que
l'identité de fabrication. La note de l'étape (`index.html:295-297`) se réécrit :

```html
      <h2>Livrables</h2>
      <p class="note">Chaque livrable compose son propre intérieur, donc sa propre
        pagination, donc son propre dos et sa propre planche. Les fichiers sont écrits à côté
        du <code>.ozalid</code>, dans un répertoire par livrable.</p>
```

et le pointeur du pied (`index.html:492`) :

```html
<footer id="pied">
  <label id="visee"><span>Vu pour</span><select id="inLivrable"></select></label>
```

- [x] **Étape 3 : Renommer dans `tests/`**

Les mêmes identifiants, dans `packages.test.js` (54 points), `coquille.test.js` (29),
`composition.test.js` (8), `couverture.test.js` (3). **Aucune assertion ne change** : ce
sont les mêmes tests sur les mêmes gestes.

- [x] **Étape 4 : Lancer la suite pour la voir passer, à l'identique**

Depuis la racine : `node --test tests/*.test.js`.

Attendu : **le même nombre de tests passés qu'à l'étape 1**, 0 échec. Un nombre différent
signale un test perdu en route, pas un renommage réussi. Si la suite ne rend pas la main,
c'est un renommage à moitié fait (piège transverse).

- [x] **Étape 5 : Recaler le README**

Dans `README.md`, réécrire la section « Le prestataire, choisi une seule fois »
(l. 278-300) : le titre devient « Le livrable, choisi une seule fois », « destinataires »
devient « livrables », et le premier paragraphe dit l'identité à quatre axes plutôt que le
prestataire. Le reste de la section — les relevés vides, la vignette de planche — est
vrai tel quel et ne bouge pas.

- [x] **Étape 6 : Commit**

```bash
git add src/ tests/ README.md
git commit -m "L'écran nomme des livrables, et le README avec lui"
```

---

### Tâche 8 : La spec rejoint ce qui a été fait

**Fichiers :**
- Modifier : `docs/superpowers/specs/2026-08-26-catalogue-et-livrables-design.md` (§ 6, § 10)
- Modifier : `docs/superpowers/plans/2026-08-26-catalogue-lot-3-la-cascade.md` (les cases)

- [x] **Étape 1 : Recaler le § 6**

Trois faits que l'exécution a précisés, à écrire dans la spec :

1. La finition ne paraît que chez un POD qui en déclare — aucun des six fournis n'en
   déclare aujourd'hui, et le contrôle attend le lot 4.
2. Le relevé de dos suit le papier retenu, non le papier d'office du POD.
3. Le POD et le format ne se règlent pas : ils se choisissent à l'ajout. La reliure, si.
   La phrase « trois réglages dessus » est juste ; ce qu'elle implique du gabarit ne
   l'était pas, et le lot 2 avait tranché l'inverse.
4. « chacun limité à ce que ce POD offre **pour ce format** » promet plus que le modèle
   ne porte : reliures, finitions et papiers vivent sur le POD, jamais sous un format
   (`Pod`, `catalogue.rs:190-202`), et le lot 1 l'a voulu ainsi — « un arbre POD > format
   > reliure > papier aurait obligé à recopier les quatre papiers d'un POD sous chacun de
   ses formats » (`catalogue.rs:19-22`). Aucune exception par format n'est déclarable
   aujourd'hui. La phrase devient « ce que ce POD offre » ; le jour où un POD offrira un
   papier sous un seul de ses formats, ce sera un chantier de catalogue, pas d'écran.

- [x] **Étape 2 : Cocher le lot 3 au § 10**

- [x] **Étape 3 : Commit**

```bash
git add docs/
git commit -m "La spec dit ce que la cascade fait, et le lot 3 est coché"
```

---

## À l'œil, avant de clore le lot

> ✅ **Les sept faites et validées par l'utilisateur le 26/08**, au moyen du POD d'essai
> `essai-deux-reliures.toml` déposé dans
> `~/Library/Application Support/cloud.gavini.ozalid/pods/` — deux formats, deux reliures
> composables, une non outillée, deux finitions, et deux papiers dont un seul publie sa
> formule de dos. Aucun POD fourni n'offrant deux reliures composables, ce fichier est le
> seul moyen d'exercer la reliure réglable à l'écran. Il ferme du même coup la
> **vérification 4 du lot 2**, en suspens depuis sa clôture : un `.toml` déposé sur le
> poste paraît sans recompilation.
>
> Reste ouverte la **vérification 5 du lot 2** : réécrire une marge de ce fichier, rouvrir
> un livre déjà composé chez « Essai », et voir le dos se déclarer périmé.

Dans l'application, sur un vrai livre :

1. **La cascade** : ajouter un livrable chez KDP en 5 × 8, vérifier que la liste des
   formats a bien suivi le changement de POD et que le livrable ajouté est celui qu'on a
   désigné.
2. **Le grisé** : chez BoD, la couverture rigide paraît grisée, sa raison lisible sous la
   ligne, et le clic dessus ne la choisit pas.
3. **La disposition à 1040 × 780** : la liste des vérifications est à la tâche 5, étape 2.

Les deux dernières demandent un POD à **deux reliures composables**, qu'aucun des six
fournis n'offre — BoD en a deux, mais l'une n'est pas outillée. Elles passent donc par un
fichier déposé sur le poste, ce qui exerce du même coup la vérification 4 restée en
suspens au lot 2. Déposer dans `~/Library/Application Support/<app>/pods/essai.toml` un
POD à deux reliures outillées (chacune avec sa `geometrie`, sa `pages` et sa `parite`,
que `verifie_reliure` réclame), relancer l'application, puis :

4. **La reliure réglée** : changer la reliure d'un livrable et vérifier que le pied passe
   à « dos non composé » — le gabarit a changé, la mesure de l'ancien ne vaut plus pour
   lui — puis que recomposer le renseigne.
5. **Le pointeur du pied** : deux livrables de ce POD, même format et même papier, l'un
   dans chaque reliure, se lisent différemment dans « Vu pour ».

Les vérifications à l'œil **4 et 5 du lot 2** restent en suspens et ne sont pas reprises
ici : elles portent sur les fichiers déposés dans `<config>/pods/`, pas sur l'écran.

## Ce que ce lot ne fait pas

- **Il ne relève aucune donnée chez un POD.** Ni finitions, ni formats, ni papiers : c'est
  le lot 4, et chaque valeur y viendra avec sa `source`.
- **Il ne grise pas un POD entier.** Un POD sans reliure composable ne paraît pas dans la
  liste d'ajout, comme aujourd'hui. Aucun POD fourni n'est dans ce cas.
- **Il ne touche pas à la politique d'invalidation des mesures.** `normalise` continue de
  ne tourner qu'à l'ouverture ; une mesure orpheline y est élaguée, et personne ne la lit
  d'ici là (tâche 2, étape 5).
- **Il ne réécrit pas `docs/COOKBOOK.md`**, qui pointe encore deux fois vers `providers.rs`
  — chemin mort. C'est une dette du lot 4.
