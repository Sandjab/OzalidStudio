# Livraison refondue, lot 3 — l'écran

> **Pour un exécutant agentique :** SOUS-COMPÉTENCE REQUISE : `superpowers:subagent-driven-development`
> (recommandé) ou `superpowers:executing-plans` pour exécuter ce plan tâche par tâche. Les
> étapes sont des cases à cocher (`- [x]`).

**But :** l'écran du § 5 de la spec — un formulaire à cinq listes et deux verbes, une ligne
par livrable qui porte son propre compte rendu, des groupes par imprimeur, quatre boutons par
ligne — et la disparition des trois commandes dont les gestes n'existent plus. C'est le
dernier lot du chantier : à sa fin, un livrable se génère, se modifie, se régénère et se
supprime depuis sa ligne, et l'écran dit où en est chacun sans qu'on ait à composer pour le
savoir.

**Architecture :** l'état calculé au lot 1 (`empreinte::Etat`) descend enfin dans la vue —
`livraison_vue` reçoit le `Projet` au lieu de la seule `Livraison`, ce qui était prévu et
noté au lot 2. La vignette, elle, ne descend **pas** dans la vue : elle passe par une commande
dédiée sur le modèle d'`envoi_vignettes`, parce que `vue()` est rendue par toute commande qui
écrit et qu'un base64 par livrable à chaque frappe serait payé pour rien. Côté front,
`livraison.js` est réécrit autour de deux fonctions : le formulaire (cinq listes, deux verbes)
et la ligne (le corps de l'ancien compte rendu, groupé par imprimeur).

**Pile :** Rust 2021, `serde` ; front vanilla sans bundler. Tests : `cargo test` depuis
`src-tauri/`, `node --test tests/*.test.js` depuis la racine, `cargo run --example temoin`
comme témoin.

**Spec :** `docs/superpowers/specs/2026-08-29-livraison-refondue-design.md` (§ 5, § 6, § 7).
**Reconnaissance :** `docs/superpowers/2026-08-30-reconnaissance-livraison-lot-3.md` — les
verdicts cités ici (1a à 7) y sont, chacun appuyé sur un fichier et une ligne.

---

## Décisions arbitrées (30/08) — ne pas les rouvrir

Deux arbitrages de produit, pris avec l'utilisateur après lecture de la reconnaissance :

1. **Une ligne rouverte se remplit du modèle, plus sa vignette relue du disque.** Pages,
   gouttière, dos recalculé, état, chemins dérivés du nom, vignette. Ce que seule la
   composition a vu — dos rogné, avertissements, polices de repli, intérieur partagé — ne
   paraît que dans la session qui a généré. **Le format du `.ozalid` ne bouge pas** : ni
   `VERSION`, ni champ neuf. (Verdict 3d de la reconnaissance ; la troisième voie — retenir
   le compte rendu dans le fichier — a été écartée.)
2. **Les relevés passent par Modifier → Remplacer.** `livrable_regler` disparaît sans
   remplaçant : un relevé corrigé change le dos, donc la planche, donc le package d'avant est
   faux, et recomposer est ce qu'il faut. Conséquence à tenir : **ouvrir Modifier doit
   reprendre les relevés déjà saisis**, sans quoi le geste devient une ressaisie.

Et trois décisions techniques qui découlent de la reconnaissance :

3. **`Etat` perd son `Copy`.** `Etat::Echec` doit porter le message que
   `Generation::Echec { message }` retient (`projet.rs:311-313`) et qu'`etat()` laisse tomber
   aujourd'hui (`empreinte.rs:130`) : une ligne qui dit « échec » sans dire lequel oblige à
   régénérer pour savoir. Un `String` dans la variante casse `Copy` — c'est admis, le type
   n'est jamais copié en boucle chaude.
4. **`normalise` remet le livrable en `Generation::Jamais` quand il replie le papier.**
   Corriger la donnée, pas son affichage (verdict 4a).
5. **Le faux backend de `coquille.test.js` apprend les quatre verbes avant toute autre tâche
   du front.** C'est un préalable, pas une conséquence (verdict 6b).

## Contraintes globales

- **Français** dans l'interface, les commentaires, les messages et les commits ; termes
  techniques anglais conservés tels quels (`chunk`, `viewport`, `canvas`).
- **Aucun test neuf ne compte s'il n'a pas été vu échouer.** TDD strict, ou mutation ciblée.
- `VERSION` du `.ozalid` **ne change pas**, et aucun champ n'entre dans le modèle : décision 1.
- Le témoin doit valoir **le même compte de pages qu'avant le lot** — 98 / 118 / 100.
- **Ne bougent pas** (spec § 5) : le pied « Vu pour », la génération d'ebooks, la vignette et
  sa largeur, le fichier de sortie (mêmes noms, même répertoire, même fiche de téléversement).
- La liste des livrables garde son ordre : les groupes se rangent dans l'ordre du premier
  ajout, les lignes dans leur groupe de même. **Aucun tri** — un regroupement stable.

## Avant chaque commit

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
cd src-tauri && cargo test
cd .. && node --test tests/*.test.js
cd src-tauri && cargo run --example temoin     # dès qu'un fichier de src-tauri/ a bougé
```

`clippy` est rouge sur la baseline depuis rustc 1.98 — `police.rs:123` et
`examples/packager.rs:32`, lint `chunks_exact_to_as_chunks`. Ce sont les deux seuls
avertissements admis ; tout autre est de votre fait.

**Le front est embarqué dans le binaire à la compilation.** Après un changement de `src/`
seul, `touch src-tauri/src/lib.rs` avant `cargo build`, sinon le binaire garde l'ancien front.
Voir les pièges du `CLAUDE.md` — celui des icônes et celui des ressources `include_str!` ne
mordent pas ici, mais celui du front, si.

Baseline relevée le 30/08 avant d'écrire ce plan : `cargo test` **639 passés, 0 échec,
11 ignorés** ; `node --test` **305 passés, 0 échec**.

## Structure des fichiers

| fichier | rôle |
|---|---|
| `src-tauri/src/empreinte.rs` | **modifié** — `Etat::Echec` porte son message, `Etat` perd `Copy` |
| `src-tauri/src/projet.rs` | **modifié** — `normalise` défait la génération quand il replie le papier |
| `src-tauri/src/commands.rs` | **modifié** — `livraison_vue(projet)`, `LivrableVue.etat`, `livrable_vignettes`, puis suppression des trois commandes |
| `src-tauri/src/lib.rs` | **modifié** — l'`invoke_handler` perd trois entrées, en gagne une |
| `src/index.html` | **modifié** — le formulaire à cinq listes, la zone `#packages` retirée |
| `src/livraison.js` | **modifié** — le formulaire et la ligne ; `reglerLivrable` et l'ajout partent |
| `src/app.js` | **modifié** — les branchements, un libellé de ligne à côté de `libelleLivrable` |
| `src/styles.css` | **modifié** — le groupe, la ligne-compte rendu |
| `tests/coquille.test.js` | **modifié** — le faux backend apprend les quatre verbes |
| `tests/packages.test.js` | **modifié** — repris test par test |
| `README.md` | **modifié** — la section « 3 · Livraison » |

## Ce que la spec réclame, et la tâche qui le porte

Relu point par point sur le § 5, le § 6 et le § 7 après avoir écrit le plan. Rien du § 5 n'est
sans tâche.

| ce que la spec demande | § | tâche |
|---|---|---|
| le formulaire à cinq listes | 5 | 5 |
| les relevés sous les listes, si l'imprimeur en exige | 5 | 5 |
| « Générer », le verbe courant | 5 | 5 |
| « Tout regénérer », global, en tête d'étape | 5 | 8 |
| le groupe porte l'imprimeur, la ligne ne le répète plus | 5 | 6 |
| les groupes dans l'ordre du premier ajout, les lignes de même | 5 | 6 |
| les groupes ne se replient pas | 5 | 6 (rien à écrire : aucun pli) |
| le corps du compte rendu devient la ligne | 5 | 6 |
| le marquage de péremption sur la ligne | 5 | 1 (le champ), 6 (l'affichage) |
| les alertes descendent sur la ligne qu'elles concernent | 5 | 6 |
| les quatre boutons par ligne | 5 | 7 |
| l'attente garde son dispositif | 5 | 5, 7, 8 |
| la zone intermédiaire disparaît | 5 | 8 |
| ne bougent pas : pied, ebooks, vignette et sa largeur | 5 | contraintes globales |
| `livrable_ajouter`, `_regler`, `_retirer` disparaissent | 6 | 9 |
| `packager` inchangée dans son principe | 6 | 8 |
| `livrable_viser` inchangée | 6 | aucune — rien à faire |
| le README réécrit sur le nouveau | 7 | 10 |
| les `.ozalid` existants s'ouvrent sans conversion | 7 | 2 (et le témoin de chaque tâche) |
| le fichier de sortie ne change pas | 7 | contraintes globales |

Deux exigences du § 9 que ce lot doit **vérifier sans les écrire** : elles sont tenues depuis
les lots 1 et 2, et rougiraient si une tâche les cassait — « une couverture retouchée périme
la couverture, et elle seule » (`empreinte.rs`) et « un `.ozalid` d'avant s'ouvre en *jamais
généré*, ses relevés intacts » (`projet.rs`). La tâche 2 touche `normalise` : les relire
après elle.

Aucun fichier créé. `livraison.js` reste un seul fichier : il fera environ la taille
d'aujourd'hui — le formulaire et la ligne remplacent la ligne-formulaire et le compte rendu —
et le découper séparerait deux moitiés d'un même écran qui changent ensemble.

---

### Tâche 1 : L'état d'un livrable descend jusqu'à l'écran

Le lot 1 a rendu l'état calculable, le lot 2 devait le faire descendre et ne l'a pas fait —
le commentaire d'`Etat` le dit encore (`empreinte.rs:110-111`). Sans ce champ, aucune ligne du
§ 5 ne peut s'écrire. Le message de l'échec descend avec : une ligne qui dit « échec » sans
dire lequel oblige à régénérer pour savoir.

**Fichiers :**
- Modifier : `src-tauri/src/empreinte.rs` (`Etat`, `etat`)
- Modifier : `src-tauri/src/commands.rs` (`LivrableVue`, `livraison_vue`, son appel `:2791`,
  le test `:3157`)

**Interfaces :**
- Consomme : `projet::Generation`, `empreinte::{interieur, couverture}`.
- Produit : `enum Etat { Jamais, Echec { message: String }, AJour, Perime { interieur: bool,
  couverture: bool } }` — sans `Copy` ; `fn livraison_vue(projet: &Projet) -> LivraisonVue` ;
  `LivrableVue.etat: Etat`, sérialisé `{"etat":"jamais"|"echec"|"ajour"|"perime", …}`.

- [ ] **Étape 1 : écrire les tests qui échouent**

Dans `src-tauri/src/empreinte.rs`, module `tests` :

```rust
/// Le message de l'échec voyage avec l'état, sinon la ligne dit « échec » sans dire
/// lequel — et il faut régénérer pour l'apprendre, c'est-à-dire refaire la chose qui a
/// échoué pour savoir pourquoi elle a échoué.
#[test]
fn un_echec_porte_sa_raison() {
    let (projet, mut l) = projet_a_un_livrable();
    l.generation = crate::projet::Generation::Echec {
        message: "dos non relevé sur le gabarit".into(),
    };
    assert_eq!(
        etat(&projet, &l),
        Etat::Echec { message: "dos non relevé sur le gabarit".into() }
    );
}
```

Dans `src-tauri/src/commands.rs`, module `tests` :

```rust
/// La vue porte l'état, sans quoi l'écran ne peut rien dire d'un livrable qu'il n'a pas
/// lui-même généré dans la session courante.
#[test]
fn la_vue_porte_l_etat_de_chaque_livrable() {
    let projet = projet_a_deux_livrables();
    let v = livraison_vue(&projet);
    assert_eq!(v.livrables.len(), 2);
    assert!(matches!(v.livrables[0].etat, crate::empreinte::Etat::Jamais));
}

/// Le champ voyage sous la forme que le front lit : un objet étiqueté, et les deux
/// drapeaux de la péremption. C'est cette forme-là que `livraison.js` consomme, et la
/// geler ici évite qu'un `rename` la change sans que rien ne rougisse.
#[test]
fn l_etat_se_serialise_comme_le_front_le_lit() {
    let e = crate::empreinte::Etat::Perime { interieur: false, couverture: true };
    assert_eq!(
        serde_json::to_string(&e).unwrap(),
        r#"{"etat":"perime","interieur":false,"couverture":true}"#
    );
    let j = crate::empreinte::Etat::Jamais;
    assert_eq!(serde_json::to_string(&j).unwrap(), r#"{"etat":"jamais"}"#);
}
```

Les deux helpers `projet_a_un_livrable` et `projet_a_deux_livrables` : réutiliser ceux du
module de tests s'ils existent — les chercher avant d'en écrire — sinon les monter sur le
patron du test `commands.rs:3157`, en construisant un `Projet` complet plutôt qu'une
`Livraison` seule.

- [ ] **Étape 2 : voir les tests échouer**

```bash
cd src-tauri && cargo test empreinte::tests::un_echec_porte_sa_raison \
  commands::tests::la_vue_porte_l_etat commands::tests::l_etat_se_serialise
```

Attendu : échec de compilation — `Etat::Echec` ne prend pas de champ, `LivrableVue` n'a pas
de champ `etat`, `livraison_vue` ne prend pas un `&Projet`. **C'est un échec valable** : le
test ne peut pas compiler contre une API qui n'existe pas. Le voir est ce qui prouve qu'il
mord.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `empreinte.rs`, la variante et le `derive` :

```rust
/// Où en est un livrable, comparé à l'état courant du projet.
///
/// Pas de `Copy` : `Echec` porte le message de la génération ratée. Le type ne circule
/// qu'une fois par livrable et par vue — le clone se paie moins cher qu'une ligne qui
/// dirait « échec » sans dire lequel.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "etat", rename_all = "lowercase")]
pub enum Etat {
    /// Jamais généré : rien à regarder, rien à refaire tant qu'on ne l'a pas demandé.
    Jamais,
    /// La dernière génération a échoué, et dit pourquoi.
    Echec { message: String },
    AJour,
    Perime { interieur: bool, couverture: bool },
}
```

et dans `etat()`, la branche qui laissait tomber le message :

```rust
crate::projet::Generation::Echec { message } => {
    return Etat::Echec { message: message.clone() }
}
```

Dans `commands.rs`, le champ et la signature :

```rust
pub struct LivrableVue {
    // … les champs existants, inchangés …
    compose: Option<MesureVue>,
    /// Où en est le package de ce livrable, comparé à l'état courant du projet. C'est ce
    /// que la ligne montre sans avoir rien composé — et la seule chose qui distingue
    /// « à jour » de « il faudrait regénérer ».
    etat: crate::empreinte::Etat,
}
```

```rust
/// La livraison telle que le front la lit : un livrable par livrable, son identité à
/// quatre axes, la mesure de **son gabarit** — que deux papiers partagent, chacun en
/// tirant son propre dos — et son état de génération.
///
/// Le `Projet` entier et non la seule `Livraison` : `empreinte::etat` compare le livrable
/// au manuscrit, à la maquette et aux images, qui n'appartiennent pas à la livraison.
fn livraison_vue(projet: &Projet) -> LivraisonVue {
    let l = &projet.meta.livraison;
    let vue = |liv: &Livrable| -> LivrableVue {
        // … le corps existant, inchangé jusqu'au montage de LivrableVue …
        LivrableVue {
            // … les champs existants …
            compose,
            etat: crate::empreinte::etat(projet, liv),
        }
    };
    LivraisonVue {
        livrables: l.livrables.iter().map(&vue).collect(),
        courant: l.courant.clone(),
        deja_compose: l.deja_compose,
    }
}
```

L'appel de `vue()` (`commands.rs:2791`) devient `livraison: livraison_vue(&o.projet)`.

- [ ] **Étape 4 : voir les tests passer**

```bash
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

Attendu : 639 + 3 passés, 0 échec. Si un test existant rougit sur `Etat` qui n'est plus
`Copy`, remplacer le déplacement par un `.clone()` à l'appel — **jamais** en remettant `Copy`.

- [ ] **Étape 5 : le témoin, puis commit**

```bash
cd src-tauri && cargo run --example temoin      # attendu : 98 / 118 / 100
git add src-tauri/src/empreinte.rs src-tauri/src/commands.rs
git commit -m "L'état d'un livrable, et la raison de son échec, descendent dans la vue"
```

---

### Tâche 2 : Un papier replié à l'ouverture n'a plus de package

`normalise` remplace le papier par le premier du POD quand le catalogue ne porte plus celui du
fichier (`projet.rs:538`). La clé à quatre axes change, donc le répertoire — il est nommé par
elle (`package.rs:612`) —, mais `l.generation` reste. Or `empreinte::couverture` inclut le
papier (`empreinte.rs:80`) et `empreinte::interieur` ne l'inclut pas : l'écran dirait
« périmé : couverture » sur un livrable dont **tout** le package a disparu. Corriger la
donnée, pas son affichage.

**Fichiers :**
- Modifier : `src-tauri/src/projet.rs` (`normalise`, la branche du repli, `:537-548`)

**Interfaces :**
- Consomme : `projet::Generation`.
- Produit : rien de neuf ; un invariant — après `normalise`, un livrable dont le papier a été
  replié est en `Generation::Jamais`.

- [ ] **Étape 1 : écrire le test qui échoue**

Dans `src-tauri/src/projet.rs`, module `tests` :

```rust
/// **Le test qui protège la ligne du lot 3.** Le repli de papier renomme le livrable, donc
/// son répertoire : les fichiers de la génération d'avant sont sous une clé que plus
/// personne ne porte. Garder l'empreinte ferait dire à l'écran « périmé : couverture » là
/// où la vérité est « il n'y a plus de package ». Régénérer réécrirait tout de toute
/// façon — mais l'utilisateur aurait lu, entre-temps, qu'il ne manquait qu'une couverture.
#[test]
fn un_papier_replie_perd_sa_generation() {
    let mut l = livraison_d_un_livrable_papier_inconnu();
    l.livrables[0].generation = Generation::Fait {
        interieur: "aaaaaaaaaaaaaaaa".into(),
        couverture: "bbbbbbbbbbbbbbbb".into(),
    };
    let avant = l.livrables[0].cle();
    l.normalise();
    assert_ne!(l.livrables[0].cle(), avant, "le papier a bien été replié");
    assert_eq!(l.livrables[0].generation, Generation::Jamais);
}

/// Le pendant : un livrable que `normalise` ne touche pas garde ce qu'il avait. Sans ce
/// test, remettre `Jamais` à tout le monde passerait le test ci-dessus et périmerait
/// silencieusement tous les packages à chaque ouverture.
#[test]
fn un_livrable_intact_garde_sa_generation() {
    let mut l = livraison_d_un_livrable_valide();
    let faite = Generation::Fait {
        interieur: "aaaaaaaaaaaaaaaa".into(),
        couverture: "bbbbbbbbbbbbbbbb".into(),
    };
    l.livrables[0].generation = faite.clone();
    l.normalise();
    assert_eq!(l.livrables[0].generation, faite);
}
```

Les deux helpers : chercher dans le module de tests de `projet.rs` ceux qui montent déjà une
`Livraison` pour les tests d'élagage — le repli de papier y est certainement déjà éprouvé —
et s'en servir plutôt que d'en écrire de nouveaux.

- [ ] **Étape 2 : voir le premier test échouer**

```bash
cd src-tauri && cargo test projet::tests::un_papier_replie_perd_sa_generation \
  projet::tests::un_livrable_intact_garde_sa_generation
```

Attendu : `un_papier_replie_perd_sa_generation` **échoue** — `left: Fait { .. }, right: Jamais` ;
`un_livrable_intact_garde_sa_generation` passe déjà. C'est normal : le second est le garde-fou
de la correction, pas sa cible.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `normalise`, juste après le rattrapage de `courant` et avant le commentaire « La mesure
du gabarit survit » :

```rust
                if avant == vise {
                    rebaptise = Some(l.cle());
                }
                // Les fichiers de la génération d'avant sont restés sous l'ancienne clé,
                // et le répertoire est nommé par elle : ce livrable n'a plus de package.
                // Sans cette ligne, `empreinte::interieur` — qui ne dépend pas du papier —
                // le laisserait paraître à jour sur son intérieur, et l'écran ne
                // signalerait qu'une couverture périmée là où tout a disparu.
                l.generation = Generation::Jamais;
                // La mesure du gabarit survit : le papier ne pagine pas.
```

- [ ] **Étape 4 : voir les tests passer**

```bash
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

Attendu : 642 + 2 passés, 0 échec.

- [ ] **Étape 5 : le témoin, puis commit**

```bash
cd src-tauri && cargo run --example temoin      # attendu : 98 / 118 / 100
git add src-tauri/src/projet.rs
git commit -m "Un papier replié à l'ouverture perd la génération qu'il ne porte plus"
```

---

### Tâche 3 : Les vignettes des livrables se relisent du disque

La ligne du § 5 porte la vignette de sa planche, y compris à la réouverture d'un projet
(décision 1). Elle ne peut pas descendre dans `ProjetVue` : `vue()` est rendue par **toute**
commande qui écrit dans le projet, et encoder un PNG par livrable à chaque frappe qui touche
le livre se paierait pour rien. `envoi_vignettes` (`commands.rs:2409-2421`) a tranché ce cas
dans le bon sens — une commande dédiée, sans cache, demandée à l'ouverture de l'étape.

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs` (commande neuve, près de `packager`)
- Modifier : `src-tauri/src/lib.rs` (`invoke_handler`)

**Interfaces :**
- Consomme : `sorties_racine`, `package::nom`, `donnee_png`.
- Produit : `#[tauri::command] pub fn livrable_vignettes(atelier: State<Atelier>) ->
  Result<BTreeMap<String, String>, String>` — la clé du livrable vers sa vignette en
  `data:image/png;base64,…`. Les livrables sans vignette sur le disque sont **absents** de la
  table, jamais présents avec une valeur vide.

- [ ] **Étape 1 : écrire le test qui échoue**

Dans `src-tauri/src/commands.rs`, module `tests` :

```rust
/// Une vignette écrite par une génération d'hier se retrouve à la réouverture : c'est
/// tout ce qui permet à la ligne de montrer sa planche sans recomposer. Un livrable dont
/// le fichier n'est pas là est **absent** de la table — une entrée vide se lirait comme
/// une vignette illisible, alors qu'il n'y en a simplement jamais eu.
#[test]
fn les_vignettes_se_relisent_du_disque_et_les_absentes_ne_mentent_pas() {
    let tmp = tempfile::tempdir().unwrap();
    let racine = tmp.path().join("sorties");
    let cle = "lulu-108x175-broche-standard";
    let dossier = racine.join(cle);
    std::fs::create_dir_all(&dossier).unwrap();
    // Les huit octets de la signature PNG suffisent : `donnee_image` relève le type sur le
    // contenu, et le test porte sur le fait de trouver le fichier, pas sur son image.
    std::fs::write(
        dossier.join(package::nom(cle, "couverture", "png")),
        [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a],
    )
    .unwrap();

    let v = vignettes_du_disque(&racine, &[cle.to_string(), "absent-du-disque".to_string()]);
    assert!(v[cle].starts_with("data:image/png;base64,"));
    assert!(!v.contains_key("absent-du-disque"));
}
```

Le nom du fichier est vérifié, pas supposé : `package.rs:470` écrit la vignette sous
`nom(cle, "couverture", "png")` — « couverture » et non « planche », parce que le PNG est
l'aperçu du même fichier que `couverture-<clé>.pdf`. Un test qui inventerait un nom passerait
au vert contre une implémentation qui inventerait le même : il ne protégerait rien.

- [ ] **Étape 2 : voir le test échouer**

```bash
cd src-tauri && cargo test commands::tests::les_vignettes_se_relisent_du_disque
```

Attendu : échec de compilation — `vignettes_du_disque` n'existe pas.

- [ ] **Étape 3 : écrire l'implémentation**

Une fonction libre, éprouvable sans `State` ni Typst — la manière déjà prise pour
`refuse_doublon`, `reglage_refuse` et `dossiers_d_envoi` :

```rust
/// Les vignettes de planche qu'une génération a laissées sur le disque, par clé de livrable.
///
/// Un livrable sans fichier est **absent** de la table, jamais présent avec une valeur vide :
/// l'écran distingue « pas encore généré » de « vignette illisible », et une entrée creuse
/// confondrait les deux.
fn vignettes_du_disque(racine: &Path, cles: &[String]) -> BTreeMap<String, String> {
    cles.iter()
        .filter_map(|cle| {
            // `package.rs:470` : la vignette est l'aperçu de la planche, et porte donc
            // le nom de la couverture, pas celui d'une « planche » qui n'existe nulle part.
            let png = racine.join(cle).join(package::nom(cle, "couverture", "png"));
            donnee_png(&png).ok().map(|d| (cle.clone(), d))
        })
        .collect()
}

/// Les vignettes de planche des livrables du livre, pour l'affichage de l'étape.
///
/// Aucun cache, et hors de `vue()` : `vue` est rendue par toute commande qui écrit dans le
/// projet, et un base64 par livrable à chaque frappe se paierait pour rien. L'interface ne
/// demande cette table qu'à l'ouverture de l'étape et après une génération — le même
/// arbitrage qu'`envoi_vignettes`, pour la même raison.
///
/// Un projet jamais enregistré n'a pas de racine de sorties : la table est vide, et ce n'est
/// pas une erreur — il n'a rien pu générer.
#[tauri::command]
pub fn livrable_vignettes(atelier: State<Atelier>) -> Result<BTreeMap<String, String>, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let Ok(racine) = sorties_racine(o) else {
        return Ok(BTreeMap::new());
    };
    let cles: Vec<String> = o.projet.meta.livraison.livrables.iter().map(|l| l.cle()).collect();
    Ok(vignettes_du_disque(&racine, &cles))
}
```

Et dans `lib.rs`, à la suite des quatre verbes :

```rust
            commands::livrable_vignettes,
```

- [ ] **Étape 4 : voir le test passer**

```bash
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

Attendu : 644 + 1 passés, 0 échec.

- [ ] **Étape 5 : le témoin, puis commit**

```bash
cd src-tauri && cargo run --example temoin      # attendu : 98 / 118 / 100
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "Les vignettes de planche se relisent du disque, hors de la vue"
```

---
### Tâche 4 : Les deux faux backends apprennent les quatre verbes

Préalable à tout test d'écran, et **première tâche du front** : sans lui, aucun test de ligne
ne peut être écrit. Attention, il y a **deux** faux backends, pas un — `coquille.test.js` en
porte un dans un `switch` (`:228,246,256`), `packages.test.js` le sien dans une suite de `if`
(`:253-300`), avec un modèle de rétention de mesure qui lui est propre (`:188-231`). Les deux
doivent apprendre les mêmes verbes, sinon la moitié des tests d'écran ne pourra pas s'écrire.

Cette tâche ne change **aucun** comportement de l'application : elle prépare le terrain, et se
vérifie à ce que les 305 tests JS restent verts.

**Fichiers :**
- Modifier : `tests/coquille.test.js` (le `switch` du faux)
- Modifier : `tests/packages.test.js` (le faux `invoke` de `ouvre`)

**Interfaces :**
- Consomme : les commandes des tâches 1 et 3 — `livrable_generer`, `livrable_remplacer`,
  `livrable_regenerer`, `livrable_supprimer`, `livrable_vignettes`.
- Produit : dans les deux faux, une réponse pour chacune, et un champ `etat` sur chaque
  livrable de la vue.

- [ ] **Étape 1 : écrire le test qui échoue**

Dans `tests/packages.test.js` :

```javascript
/**
 * Le faux backend doit répondre aux quatre verbes du lot 2, sans quoi aucun test de ligne
 * ne peut s'écrire. Ce test ne vérifie pas l'écran : il vérifie le harnais qui permettra de
 * le vérifier. C'est le seul de ce fichier dans ce cas, et c'est assumé.
 */
test('le faux backend sert les quatre verbes et l\'état de chaque livrable', async () => {
  const { invoke, projet } = await ouvre([LULU]);
  assert.strictEqual(projet().livraison.livrables[0].etat.etat, 'jamais');
  const r = await invoke('livrable_generer', { livrable: chez(LULU) });
  assert.ok(r.projet, 'générer rend la vue du projet');
  assert.ok(Array.isArray(r.packages), 'et les packages composés');
  assert.strictEqual(
    r.projet.livraison.livrables.at(-1).etat.etat, 'ajour',
    'un livrable qui vient d\'être généré est à jour'
  );
});
```

`ouvre` doit rendre `invoke` et `projet` en plus de `els` : les vérifier dans son `return`
(`packages.test.js:172-176` et la fin de la fonction) et les ajouter si besoin.

- [ ] **Étape 2 : voir le test échouer**

```bash
node --test tests/packages.test.js
```

Attendu : échec — `projet().livraison.livrables[0].etat` est `undefined`, puis
`r.projet` l'est aussi (le faux retombe sur son `default`).

- [ ] **Étape 3 : écrire l'implémentation**

Dans les **deux** fichiers, chaque livrable de la vue reçoit un `etat`. Le faux modélise la
règle du Rust, il ne la recopie pas : un livrable est `ajour` si une génération l'a touché
depuis la dernière modification, `jamais` sinon.

Dans `tests/packages.test.js`, à l'intérieur de `ouvre` :

```javascript
  // L'état que le Rust calcule par empreintes, modélisé par le plus simple qui en garde le
  // sens : générer met à jour, tout ce qui pagine périme. Le faux ne hache rien — les
  // empreintes sont éprouvées côté Rust, et les redire ici en ferait deux versions à tenir.
  const etatPose = (cles, etat) => maj({
    livrables: projet.livraison.livrables.map((d) => (
      cles.includes(d.cle) ? { ...d, etat: { etat } } : d)),
  });
```

et, dans le faux `invoke`, à la suite des `if` existants :

```javascript
    if (cmd === 'livrable_generer') {
      const f = args.livrable;
      const cle = `${f.pod}-${f.format}-${f.reliure}-${f.papier}`;
      const p = providers.find((x) => x.cle === `${f.pod}-${f.format}-${f.reliure}`);
      maj({ livrables: [...projet.livraison.livrables,
        { ...dest(p), ...f, cle, etat: { etat: 'jamais' } }] });
      const packages = sur.packages ?? [];
      etatPose([cle], 'ajour');
      return { projet: retenirPackages(packages), packages };
    }
    if (cmd === 'livrable_regenerer') {
      const packages = sur.packages ?? [];
      etatPose([args.cle], 'ajour');
      return { projet: retenirPackages(packages), packages };
    }
    if (cmd === 'livrable_remplacer') {
      const f = args.livrable;
      const cle = `${f.pod}-${f.format}-${f.reliure}-${f.papier}`;
      maj({ livrables: projet.livraison.livrables.map((d) => (
        d.cle === args.cle ? { ...d, ...f, cle, etat: { etat: 'ajour' } } : d)) });
      const packages = sur.packages ?? [];
      return { projet: retenirPackages(packages), packages, ...(sur.nettoyage_echoue
        ? { nettoyage_echoue: sur.nettoyage_echoue } : {}) };
    }
    if (cmd === 'livrable_supprimer') {
      maj({ livrables: projet.livraison.livrables.filter((d) => d.cle !== args.cle) });
      return { projet, nettoyage: sur.nettoyage
        ?? { absents: [], etrangers: [], dossier_retire: true } };
    }
    if (cmd === 'livrable_vignettes') return sur.vignettes ?? {};
```

La forme de `Nettoyage` est vérifiée, pas supposée : `package.rs:920-928` déclare
`{ absents: Vec<String>, etrangers: Vec<String>, dossier_retire: bool }` — `absents` sont les
fichiers connus qui n'étaient plus là (pas une erreur : une génération échouée n'en écrit
qu'une partie), `etrangers` ce que l'application n'a pas écrit et qui fait survivre le
répertoire. Le défaut du faux est donc le cas heureux : rien d'absent, rien d'étranger,
répertoire retiré.

Enfin, le livrable de départ naît avec un état : dans `ouvre`, la construction de `liste`
(`packages.test.js:178`) devient

```javascript
  const liste = (livrables ?? [chez(providers[0])]).map((d) => ({ etat: { etat: 'jamais' }, ...d }));
```

— l'état d'abord, pour qu'un test qui en pose un explicitement le garde.

Dans `tests/coquille.test.js`, les mêmes cinq `case`, sur le patron du `switch` existant, plus
`etat: { etat: 'jamais' }` dans `dest()` ou là où le faux monte un livrable.

- [ ] **Étape 4 : voir les tests passer**

```bash
node --test tests/*.test.js
```

Attendu : 305 + 1 passés, 0 échec. **Aucun test existant ne doit avoir changé de résultat** :
cette tâche n'ajoute que des réponses à des commandes que personne n'appelle encore.

- [ ] **Étape 5 : commit**

```bash
git add tests/coquille.test.js tests/packages.test.js
git commit -m "Les faux backends servent les quatre verbes et l'état des livrables"
```

---

### Tâche 5 : Le formulaire à cinq listes, et son second verbe

La cascade à deux listes (`livraison.js:243-268`, `index.html:386-388`) devient le formulaire
du § 5 : imprimeur, format, reliure, pelliculage, papier, puis les relevés que l'imprimeur
choisi exige, puis **Générer**. Le même formulaire sert à modifier : « Modifier » sur une
ligne le remplit et change son bouton en **Remplacer** (tâche 7 pour le bouton lui-même).

Deux choses se récupèrent telles quelles du code d'aujourd'hui, avec leur raison : la
persistance du choix entre deux ajouts (`livraison.js:250,266` — comparer deux papiers d'un
même livre est le geste pour lequel cet écran existe) et le grisé de la reliure non outillée
(`livraison.js:130-143` — le fichier tranche, une reliure porte une géométrie **ou** une
raison).

**Fichiers :**
- Modifier : `src/index.html` (la `div.ligne` de l'ajout, `:381-389`)
- Modifier : `src/livraison.js` (`afficherCascade`, `afficherFormatsDuPod` → `afficherFormulaire`)
- Modifier : `src/app.js` (branchements, `:1465-1466`)
- Modifier : `tests/packages.test.js`

**Interfaces :**
- Consomme : `pods` (l'arbre du catalogue), `providers` (la table plate), `invoke`, `tente`,
  `afficherProjet`, `h`, `$`.
- Produit : `function afficherFormulaire()` ; `function afficherAxesDuPod()` ;
  `function afficherFinition(pod)` ; `function afficherRelevesDuFormulaire(pod)` ;
  `async function genererLivrable()` ; `function lireFormulaire()` rendant
  `{ pod, format, reliure, papier, finition, dos_mm, fond_perdu_mm }` — la forme exacte que
  `livrable_generer` et `livrable_remplacer` attendent dans leur argument `livrable`.
  Identifiants de DOM : `inAjoutPod`, `inAjoutFormat`, `inAjoutReliure`, `inAjoutFinition`,
  `inAjoutPapier`, `inAjoutDos`, `inAjoutFp`, `btLivrableGenerer`, `etatLivraison`.

- [ ] **Étape 1 : écrire les tests qui échouent**

```javascript
/**
 * Les cinq listes du § 5, et l'ordre dans lequel elles se lisent : l'imprimeur commande
 * tout le reste — un format, une reliure ou un papier ne veulent rien dire sans lui.
 */
test('le formulaire offre les cinq axes du POD choisi', async () => {
  const { els } = await ouvre([LULU, KDP]);
  els.get('inAjoutPod').value = 'kdp';
  els.get('inAjoutPod').dispatchEvent(new Evenement('change'));
  assert.deepStrictEqual(
    [...els.get('inAjoutFormat').options].map((o) => o.value), ['6x9', '5x8']
  );
  assert.ok(els.get('inAjoutReliure'), 'la reliure est un choix du formulaire');
  assert.ok(els.get('inAjoutPapier'), 'le papier aussi');
});

/**
 * Le pelliculage ne paraît que là où il y en a : un contrôle vide se lit comme un choix
 * qu'on n'a pas su faire, alors qu'il n'y en avait aucun à faire. Même règle qu'à la ligne
 * d'avant le lot 3, et pour la même raison.
 */
test('le pelliculage ne paraît que chez un POD qui en déclare', async () => {
  const { els } = await ouvre([LULU, BOD]);
  els.get('inAjoutPod').value = 'lulu';
  els.get('inAjoutPod').dispatchEvent(new Evenement('change'));
  assert.strictEqual(els.get('inAjoutFinition'), undefined, 'Lulu n\'en déclare aucun');
  els.get('inAjoutPod').value = 'bod';
  els.get('inAjoutPod').dispatchEvent(new Evenement('change'));
  assert.ok(els.get('inAjoutFinition'), 'BoD en déclare trois');
});

/**
 * Le relevé suit **le papier retenu**, jamais le POD : un POD peut publier une formule de
 * dos pour l'un de ses papiers et pas pour l'autre. C'est la règle que la ligne tenait
 * (`livraison.js:174-180`) et que le formulaire reprend.
 */
test('le relevé de dos suit le papier choisi, pas l\'imprimeur', async () => {
  const { els } = await ouvre([POSTE]);
  els.get('inAjoutPapier').value = 'sans-formule';
  els.get('inAjoutPapier').dispatchEvent(new Evenement('change'));
  assert.ok(els.get('inAjoutDos'), 'ce papier ne publie pas son dos');
  els.get('inAjoutPapier').value = 'avec-formule';
  els.get('inAjoutPapier').dispatchEvent(new Evenement('change'));
  assert.strictEqual(els.get('inAjoutDos'), undefined, 'celui-ci le publie');
});

/**
 * Générer envoie les quatre axes, la finition et les relevés — la forme exacte que
 * `livrable_generer` attend. Un champ de relevé vide est une **absence**, jamais un zéro :
 * composer sur un dos nul produirait une planche fausse au lieu d'un refus.
 */
test('générer envoie le livrable entier, et un relevé vide reste une absence', async () => {
  const { els, appels } = await ouvre([POSTE]);
  els.get('btLivrableGenerer').dispatchEvent(new Evenement('click'));
  await pause();
  const [, args] = dernier(appels, 'livrable_generer');
  assert.deepStrictEqual(args.livrable, {
    pod: 'poste', format: '110x170', reliure: 'broche', papier: 'sans-formule',
    finition: null, dos_mm: null, fond_perdu_mm: null,
  });
});

/**
 * Le POD et le format retenus survivent à un réaffichage : on ajoute souvent deux
 * livrables de suite chez le même imprimeur, et comparer deux papiers d'un même livre est
 * le geste pour lequel cet écran existe. Reperdre le choix entre les deux ajouts ferait
 * payer deux clics à ce geste-là.
 */
test('le formulaire garde son imprimeur et son format d\'un ajout au suivant', async () => {
  const { els } = await ouvre([LULU, KDP]);
  els.get('inAjoutPod').value = 'kdp';
  els.get('inAjoutPod').dispatchEvent(new Evenement('change'));
  els.get('inAjoutFormat').value = '5x8';
  els.get('btLivrableGenerer').dispatchEvent(new Evenement('click'));
  await pause();
  assert.strictEqual(els.get('inAjoutPod').value, 'kdp');
  assert.strictEqual(els.get('inAjoutFormat').value, '5x8');
});
```

`Evenement`, `pause`, `dernier` et `BOD` : reprendre les helpers et les fixtures du haut de
`packages.test.js` — **les chercher avant d'en écrire**.

`POSTE` est à monter, et son nom dit ce qu'il est : **aucun des six POD fournis n'exige de
relevé**. Vérifié dans `src-tauri/pods/` — les six déclarent un `fond_perdu` et une formule de
dos par papier ; c'est ce que dit la spec § 5 (« c'est un fichier déposé sur le poste qui les
fait paraître »). La fixture modélise donc un catalogue du poste, pas TheBookEdition : le
nommer `TBE` ferait croire qu'un imprimeur fourni réclame un dos relevé, ce qui est faux et
enverrait le prochain lecteur chercher un bug dans `thebookedition.toml`. La monter sur le
patron de `LULU` (`packages.test.js:14-19` pour la table plate, `coquille.test.js:30-45` pour
l'arbre) avec **deux** papiers, l'un `dos_publie: true`, l'autre `false` : c'est ce couple qui
rend le test du relevé capable d'échouer.

- [ ] **Étape 2 : voir les tests échouer**

```bash
node --test tests/packages.test.js
```

Attendu : les cinq échouent — `inAjoutReliure`, `inAjoutPapier`, `inAjoutDos` et
`btLivrableGenerer` n'existent pas ; `dernier(appels, 'livrable_generer')` est `undefined`.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `src/index.html`, la `div.ligne` de l'ajout (`:381-389`) devient :

```html
      <!-- Le formulaire du livrable : cinq listes en cascade, l'imprimeur en tête parce
           qu'il commande tout le reste — un format, une reliure ou un papier ne veulent
           rien dire sans lui, et les mêmes 13,5 × 21,5 n'ont pas les mêmes marges chez
           deux POD. Le seul endroit de la fenêtre où des contrôles n'ont pas d'étiquette
           visible : posés contre leur bouton, ils se lisent comme le geste. Ce que l'œil
           déduit de la disposition, les `aria-label` le disent à qui ne la voit pas. -->
      <div class="ligne formulaire">
        <select id="inAjoutPod" aria-label="Imprimeur"></select>
        <select id="inAjoutFormat" aria-label="Format"></select>
        <select id="inAjoutReliure" aria-label="Reliure"></select>
        <select id="inAjoutPapier" aria-label="Papier"></select>
        <button id="btLivrableGenerer" class="primaire" type="button">Générer</button>
        <span id="etatLivraison" class="etat"></span>
      </div>
      <!-- Les relevés sous les listes, et seulement si l'imprimeur choisi en exige : ils
           dépendent du papier retenu, pas du POD. Rempli par le JS, vide autrement. -->
      <div id="ajoutReleves" class="releve"></div>
```

Le pelliculage n'est pas dans le balisage : comme la finition de la ligne d'avant, il est
créé par le JS **seulement** chez un POD qui en déclare, et inséré avant le papier.

Dans `src/livraison.js`, `afficherCascade` et `afficherFormatsDuPod` sont remplacées par :

```javascript
/**
 * Le formulaire d'un livrable : les cinq axes, puis les relevés que l'imprimeur exige.
 *
 * Les listes se reconstruisent à chaque affichage : elles ne dépendent que du catalogue,
 * qui ne bouge pas de la vie du processus, mais les reconstruire coûte cinq boucles sur
 * quelques entrées et évite d'avoir à se demander qui les a laissées dans quel état.
 *
 * Le choix retenu survit à un réaffichage, POD et format compris : on ajoute souvent deux
 * livrables de suite chez le même imprimeur. Changer de POD l'emporte de lui-même — un
 * format que le nouveau ne porte pas ne se retrouve pas.
 */
function afficherFormulaire() {
  const sel = $('inAjoutPod');
  const choisi = sel.value;
  sel.replaceChildren();
  for (const p of pods) sel.append(new Option(p.nom, p.cle));
  if (pods.some((p) => p.cle === choisi)) sel.value = choisi;
  sel.disabled = pods.length === 0;
  $('btLivrableGenerer').disabled = pods.length === 0;
  afficherAxesDuPod();
}

/**
 * Les quatre axes qui dépendent du POD choisi, et les relevés qui dépendent du papier.
 *
 * La reliure non outillée reste **visible et grisée** : le Rust la refuse déjà en citant sa
 * raison (`catalogue::resout`), et l'écran ne fait que rendre ce refus lisible avant le
 * clic. Le grisé ne se glose pas — la réserve est au README, « Limites connues » : c'est
 * une limite de l'application, pas un fait du livrable.
 */
function afficherAxesDuPod() {
  const p = pods.find((x) => x.cle === $('inAjoutPod').value);
  const garde = (sel, valeurs) => {
    const choisi = sel.value;
    sel.replaceChildren();
    for (const [cle, nom, grise] of valeurs) {
      const o = new Option(nom, cle);
      o.disabled = grise;
      sel.append(o);
    }
    if (valeurs.some(([c]) => c === choisi)) sel.value = choisi;
    sel.disabled = valeurs.length < 2;
  };
  garde($('inAjoutFormat'), (p?.formats ?? []).map((f) => [f.cle, f.nom, false]));
  garde($('inAjoutReliure'),
    (p?.reliures ?? []).map((r) => [r.cle, r.nom, r.non_outille !== null]));
  garde($('inAjoutPapier'), (p?.papiers ?? []).map((pa) => [pa.cle, pa.libelle, false]));
  afficherFinition(p);
  afficherRelevesDuFormulaire(p);
}
```

```javascript
/**
 * Le pelliculage, s'il y en a. Absent du DOM chez un POD qui n'en déclare aucun : un
 * contrôle vide se lit comme un choix qu'on n'a pas su faire, alors qu'il n'y en avait
 * aucun à faire. Cinq POD fournis sur six sont dans ce cas.
 */
function afficherFinition(p) {
  $('inAjoutFinition')?.remove();
  if (!p?.finitions.length) return;
  const sel = h('select');
  sel.id = 'inAjoutFinition';
  sel.setAttribute('aria-label', 'Pelliculage');
  // Le vide en tête : aucune finition est le cas courant, et il doit rester choisissable
  // après en avoir pris une.
  sel.append(new Option('—', ''));
  for (const fi of p.finitions) sel.append(new Option(fi.nom, fi.cle));
  $('inAjoutPapier').before(sel);
}

/**
 * Les relevés que l'imprimeur exige, sous les cinq listes.
 *
 * Le dos se réclame d'après **le papier retenu**, jamais d'après le POD : un POD peut
 * publier une formule pour l'un de ses papiers et pas pour l'autre. Le fond perdu, lui,
 * suit le gabarit — c'est la table plate qui sait le dire.
 *
 * Aucun des six POD fournis n'en exige : ce bloc reste vide sur un poste ordinaire, et ne
 * paraît que pour un catalogue déposé à la main.
 */
function afficherRelevesDuFormulaire(p) {
  const box = $('ajoutReleves');
  box.replaceChildren();
  const papier = p?.papiers.find((pa) => pa.cle === $('inAjoutPapier').value);
  const gabarit = providers.find(
    (x) => x.cle === `${$('inAjoutPod').value}-${$('inAjoutFormat').value}`
      + `-${$('inAjoutReliure').value}`
  );
  if (papier && !papier.dos_publie) {
    box.append(champReleve('inAjoutDos', 'Dos relevé (mm)', null));
  }
  if (gabarit?.fond_perdu === null) {
    box.append(champReleve('inAjoutFp', 'FP (mm)', null));
  }
}
```

`champReleve` existe déjà (`livraison.js:326`) : lui retirer son argument `livrable` et son
écouteur `change`. Dans le formulaire, un relevé ne part au projet qu'au clic sur Générer — il
n'y a plus de commande d'écriture directe à qui l'envoyer (décision 2).

```javascript
/**
 * Le livrable que le formulaire décrit, dans la forme exacte que les deux verbes attendent.
 *
 * Un champ vide est une absence de relevé, pas un zéro : composer sur un dos nul produirait
 * une planche fausse au lieu d'un refus. Un contrôle absent — le pelliculage chez un POD
 * qui n'en déclare aucun — vaut `null`, pas une chaîne vide.
 */
function lireFormulaire() {
  const lu = (id) => {
    const v = $(id)?.value.trim();
    return v ? Number(v) : null;
  };
  return {
    pod: $('inAjoutPod').value,
    format: $('inAjoutFormat').value,
    reliure: $('inAjoutReliure').value,
    papier: $('inAjoutPapier').value,
    // La chaîne vide du choix « — » est une absence, pas une finition nommée.
    finition: $('inAjoutFinition')?.value || null,
    dos_mm: lu('inAjoutDos'),
    fond_perdu_mm: lu('inAjoutFp'),
  };
}

/**
 * Générer : pose le livrable et compose, d'un seul geste.
 *
 * L'attente garde le dispositif de `packager` — bouton éteint et ligne d'état — parce que
 * le temps d'attente ne disparaît pas, il se déplace sur chaque ajout (spec § 8).
 */
async function genererLivrable() {
  const bt = $('btLivrableGenerer');
  bt.disabled = true;
  $('etatLivraison').className = 'etat';
  $('etatLivraison').textContent = 'composition du package…';
  try {
    const r = await invoke('livrable_generer', { livrable: lireFormulaire() });
    afficherProjet(r.projet);
    retenirPackagesDeLaSession(r.packages);
    $('etatLivraison').textContent = '';
  } catch (e) {
    $('etatLivraison').textContent = String(e);
    $('etatLivraison').className = 'etat erreur';
  } finally {
    bt.disabled = false;
  }
}
```

`retenirPackagesDeLaSession` est posée à la tâche 6 : elle range les `Resultat` par clé pour
que la ligne montre ce que seule la composition a vu. Ici, l'appeler suffit.

Dans `src/app.js`, les branchements (`:1465-1466`) :

```javascript
$('inAjoutPod').addEventListener('change', afficherAxesDuPod);
$('inAjoutPapier').addEventListener('change', () => afficherRelevesDuFormulaire(
  pods.find((x) => x.cle === $('inAjoutPod').value)));
$('btLivrableGenerer').addEventListener('click', genererLivrable);
```

et l'appel à `afficherCascade()` en fin d'`afficherLivrables` (`livraison.js:241`) devient
`afficherFormulaire()`.

- [ ] **Étape 4 : voir les tests passer**

```bash
node --test tests/*.test.js
```

Attendu : les cinq passent. Les tests de l'ancienne cascade
(`packages.test.js:368,398,410,431`) rougissent : ils portent sur `btAjouterLivrable`, qui
n'existe plus. **Les reprendre un par un** — leur intention survit, leur sélecteur non — et
les réécrire contre le formulaire neuf. Ne pas les supprimer : « la liste d'ajout garde les
gabarits déjà déclarés » et « le même livrable deux fois est refusé » valent toujours.

- [ ] **Étape 5 : commit**

```bash
git add src/index.html src/livraison.js src/app.js tests/packages.test.js
git commit -m "Le formulaire porte les cinq axes du livrable, et le verbe qui compose"
```

---
### Tâche 6 : La ligne porte son compte rendu, et les groupes portent leur imprimeur

Le cœur du lot. Ce qui était un bloc de compte rendu sous la liste (`afficherPackages`,
`livraison.js:390-462`) devient le corps de chaque ligne ; les lignes se rangent en groupes,
un par imprimeur, dans l'ordre du premier ajout. Le groupe porte le nom de l'imprimeur, **la
ligne ne le répète plus** — c'est ce qui règle la répétition constatée : trois TheBookEdition
ne diffèrent plus à l'écran que par ce qui les distingue vraiment.

Ce qu'une ligne montre dépend de ce qu'on sait d'elle (décision 1) : le modèle donne toujours
les pages, la gouttière, le dos, l'état et les chemins ; la vignette vient de
`livrable_vignettes` ; le dos rogné, les avertissements, les polices de repli et l'intérêt
partagé ne paraissent que dans la session qui a généré.

**Fichiers :**
- Modifier : `src/livraison.js` (`afficherLivrables` réécrite, `afficherPackages` fondue dedans)
- Modifier : `src/app.js` (un libellé de ligne, à côté de `libelleLivrable`)
- Modifier : `src/styles.css` (le groupe, la ligne)
- Modifier : `tests/packages.test.js`

**Interfaces :**
- Consomme : `projet.livraison.livrables[].etat`, `livrable_vignettes`, `cheminsGroupes`
  (`livraison.js:383`, inchangée), `nb`, `h`, `$`.
- Produit : `function libelleDansGroupe(d)` (dans `app.js`, à côté de `libelleLivrable`) ;
  `function afficherLivrables()` réécrite ; `function ligneLivrable(d)` ;
  `function retenirPackagesDeLaSession(resultats)` et sa table `packagesDeLaSession`
  (clé de livrable → `Resultat`) ; `function noteEtat(d)` ;
  `async function chargerVignettes()`.
  **Appelée mais posée à la tâche 7** : `gestesLivrable(d)` — jusque-là, la remplacer par un
  `h('div', undefined, 'gestes')` vide pour que la tâche 6 tourne seule.
  Identifiants de DOM, par livrable : `liv-<clé>` pour la ligne, `liv-etat-<clé>`,
  `liv-mesure-<clé>` (conservé), `liv-vignette-<clé>` ; par groupe : `groupe-<pod>`.

- [ ] **Étape 1 : écrire les tests qui échouent**

```javascript
/**
 * Le groupe porte l'imprimeur, la ligne ne le répète plus. C'est la raison d'être du
 * groupement : trois livrables du même POD ne se distinguaient à l'écran que par un
 * fragment noyé dans un libellé qui redisait trois fois le même nom.
 */
test('l\'imprimeur se lit une fois par groupe, jamais sur la ligne', async () => {
  const { els } = await ouvre([LULU, LULU_GRAND], {}, {
    livrables: [chez(LULU), chez(LULU_GRAND)],
  });
  assert.strictEqual(els.get('groupe-lulu').textContent.includes('Lulu'), true);
  const ligne = els.get('liv-lulu-108x175-broche-standard');
  assert.doesNotMatch(ligne.textContent, /Lulu/, 'le nom de l\'imprimeur est au groupe');
});

/**
 * L'ordre est celui du premier ajout, et il ne se réarrange pas sous la main : un ordre
 * qui bouge fait perdre la ligne qu'on visait entre deux clics. Les groupes suivent le
 * premier livrable de chaque POD, les lignes suivent la liste.
 */
test('les groupes se rangent dans l\'ordre du premier ajout', async () => {
  const { els } = await ouvre([KDP, LULU], {}, {
    livrables: [chez(KDP), chez(LULU), chez(KDP, '5x8')],
  });
  assert.deepStrictEqual(
    els.tous('.groupe').map((g) => g.id), ['groupe-kdp', 'groupe-lulu'],
    'KDP d\'abord : son premier livrable est le premier de la liste'
  );
});

/**
 * Une péremption dit **ce qui** a bougé. « Périmé » tout court obligerait à régénérer pour
 * savoir si le manuscrit ou la maquette a changé — et les deux ne coûtent pas la même
 * chose.
 */
test('une couverture périmée le dit, et ne parle pas de l\'intérieur', async () => {
  const { els } = await ouvre([LULU], {}, {
    livrables: [{ ...chez(LULU), etat: { etat: 'perime', interieur: false, couverture: true } }],
  });
  const etat = els.get('liv-etat-lulu-108x175-broche-standard');
  assert.match(etat.textContent, /couverture/);
  assert.doesNotMatch(etat.textContent, /intérieur/);
  assert.match(etat.className, /alerte/, 'une péremption se voit');
});

/**
 * Un échec montre sa raison. Sans elle, la seule façon d'apprendre pourquoi la génération
 * a échoué serait de la relancer — c'est-à-dire de refaire la chose qui a échoué.
 */
test('un échec de génération porte son message sur la ligne', async () => {
  const { els } = await ouvre([LULU], {}, {
    livrables: [{ ...chez(LULU),
      etat: { etat: 'echec', message: 'dos non relevé sur le gabarit' } }],
  });
  assert.match(
    els.get('liv-etat-lulu-108x175-broche-standard').textContent,
    /dos non relevé sur le gabarit/
  );
});

/**
 * Un livrable jamais généré ne chiffre rien et ne crie rien : il n'a rien perdu, on ne lui
 * a rien demandé. C'est la nuance que l'ancien `perimees` tenait pour toute la liste à la
 * fois, et que l'état tient maintenant ligne par ligne.
 */
test('un livrable jamais généré n\'est ni périmé ni en échec', async () => {
  const { els } = await ouvre([LULU]);
  const etat = els.get('liv-etat-lulu-108x175-broche-standard');
  assert.match(etat.textContent, /jamais généré/);
  assert.doesNotMatch(etat.className, /alerte/);
});

/**
 * La vignette d'une génération d'hier se retrouve à la réouverture : c'est ce qui permet à
 * la ligne de montrer sa planche sans recomposer, et c'est tout l'intérêt de la commande
 * dédiée. Elle vient du disque, pas du compte rendu de la session.
 */
test('une ligne retrouve la vignette laissée par une génération d\'avant', async () => {
  const { els } = await ouvre([LULU], {
    vignettes: { 'lulu-108x175-broche-standard': 'data:image/png;base64,QUJD' },
  }, { livrables: [{ ...chez(LULU), etat: { etat: 'ajour' } }] });
  await pause();
  assert.strictEqual(
    els.get('liv-vignette-lulu-108x175-broche-standard').src, 'data:image/png;base64,QUJD'
  );
});

/**
 * Ce que seule la composition a vu ne paraît que dans la session qui a généré : le dos
 * rogné, les avertissements, les polices de repli. À la réouverture, la ligne se tait
 * là-dessus plutôt que d'inventer un silence rassurant sur un fichier qu'elle n'a pas vu
 * naître.
 */
test('un dos rogné se lit sur la ligne qui vient de le composer', async () => {
  const { els } = await ouvre([LULU], { packages: [PAQUET_DOS_ROGNE] });
  els.get('btLivrableGenerer').dispatchEvent(new Evenement('click'));
  await pause();
  assert.match(
    els.get('liv-lulu-108x175-broche-standard').textContent, /rogné au pli/
  );
});
```

`els.tous` : si le shim n'offre pas de sélecteur multiple, l'ajouter dans `dom_shim.js` —
c'est le seul moyen de vérifier un **ordre**, et l'ordre est une garantie du § 5.
`LULU_GRAND` et `PAQUET_DOS_ROGNE` : monter les fixtures sur celles qui existent — `LULU` pour
le provider, le package du test `packages.test.js:951` pour le dos rogné.

- [ ] **Étape 2 : voir les tests échouer**

```bash
node --test tests/packages.test.js
```

Attendu : les sept échouent — `groupe-lulu`, `liv-<clé>`, `liv-etat-<clé>` et
`liv-vignette-<clé>` n'existent pas.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `src/app.js`, à côté de `libelleLivrable` (`:617-626`) :

```javascript
/**
 * Le libellé d'un livrable **dans son groupe** : tout ce qui le distingue, sauf l'imprimeur.
 *
 * Le groupe porte l'imprimeur (spec § 5), la ligne ne le répète pas. C'est pourquoi ce
 * libellé ne peut pas être `libelleLivrable` : celui-là part de `providers`, dont le
 * `libelle` porte « POD — format », et n'inclut la reliure que chez un POD qui en offre
 * plusieurs de composables — une règle juste pour le pied, où rien ne dit l'imprimeur, et
 * fausse ici, où le groupe le dit déjà.
 *
 * Deux libellés, deux contextes, chacun motivé : les fondre en un donnerait un libellé qui
 * ment dans l'un des deux.
 */
function libelleDansGroupe(d) {
  const p = providers.find((x) => x.cle === d.gabarit);
  const pod = pods.find((x) => x.cle === d.pod);
  // Le format sans le nom du POD : `provider.libelle` vaut « Lulu — poche 108 × 175 », et
  // c'est la moitié de droite qu'on garde.
  const format = pod?.formats.find((f) => f.cle === d.format)?.nom
    ?? p?.libelle ?? d.format;
  const reliure = pod?.reliures.find((r) => r.cle === d.reliure)?.nom ?? d.reliure;
  const papier = pod?.papiers.find((x) => x.cle === d.papier)?.libelle ?? d.papier;
  const finition = pod?.finitions.find((f) => f.cle === d.finition)?.nom;
  return [format, reliure, ...(finition ? [finition] : []), papier].join(' — ');
}
```

Dans `src/livraison.js`, la table des comptes rendus de la session et la note d'état :

```javascript
/**
 * Ce que la composition a vu, par clé de livrable, pour la durée de la session.
 *
 * Le `.ozalid` retient la mesure et les empreintes, jamais le dos rogné, les
 * avertissements ni le partage d'intérieur : ceux-là ne se reconstruisent pas sans
 * composer. Ils vivent donc ici, et une ligne rouverte demain se tait là-dessus plutôt que
 * d'inventer un silence rassurant sur un fichier qu'elle n'a pas vu naître (décision 1).
 */
let packagesDeLaSession = {};

function retenirPackagesDeLaSession(resultats) {
  for (const r of resultats ?? []) packagesDeLaSession[r.cle] = r;
  afficherLivrables();
}

/**
 * Où en est le package de cette ligne, en une phrase.
 *
 * La péremption dit **ce qui** a bougé : « périmé » tout court obligerait à régénérer pour
 * apprendre si c'est le manuscrit ou la maquette, et les deux ne coûtent pas la même chose.
 * L'échec dit sa raison, pour la même raison.
 *
 * Jamais généré n'est pas une alerte : ce livrable n'a rien perdu, on ne lui a rien demandé.
 */
function noteEtat(d) {
  const p = h('p', undefined, 'note');
  p.id = `liv-etat-${d.cle}`;
  const e = d.etat ?? { etat: 'jamais' };
  if (e.etat === 'jamais') {
    p.textContent = 'jamais généré';
  } else if (e.etat === 'ajour') {
    p.textContent = 'à jour';
  } else if (e.etat === 'echec') {
    p.className = 'note alerte';
    p.textContent = `la dernière génération a échoué : ${e.message}`;
  } else {
    p.className = 'note alerte';
    const quoi = [
      ...(e.interieur ? ['le texte'] : []),
      ...(e.couverture ? ['la couverture'] : []),
    ].join(' et ');
    p.textContent = `${quoi} a changé depuis cette génération`;
  }
  return p;
}
```

et `afficherLivrables` réécrite autour du groupement :

```javascript
/**
 * Les livrables du livre, groupés par imprimeur.
 *
 * Le groupe porte l'imprimeur, la ligne ne le répète plus : trois livrables du même POD ne
 * diffèrent plus à l'écran que par ce qui les distingue vraiment. Les groupes se rangent
 * dans l'ordre du premier ajout, les lignes dans leur groupe de même — un ordre stable, qui
 * ne se réarrange pas sous la main.
 *
 * Les groupes ne se replient pas : un imprimeur porte deux ou trois livrables, pas trente,
 * et un pli serait un état de plus à tenir pour un gain qu'on ne mesure pas.
 */
function afficherLivrables() {
  const box = $('livrables');
  box.replaceChildren();
  const declares = projet.livraison.livrables;
  // L'ordre du premier ajout, sans tri : `Map` garde l'ordre d'insertion, et le premier
  // livrable d'un POD est celui qui pose son groupe.
  const groupes = new Map();
  for (const d of declares) {
    if (!groupes.has(d.pod)) groupes.set(d.pod, []);
    groupes.get(d.pod).push(d);
  }
  for (const [pod, lignes] of groupes) {
    const bloc = h('div', undefined, 'groupe');
    bloc.id = `groupe-${pod}`;
    bloc.append(h('h3', pods.find((p) => p.cle === pod)?.nom ?? pod));
    for (const d of lignes) bloc.append(ligneLivrable(d));
    box.append(bloc);
  }
  afficherFormulaire();
  chargerVignettes();
}
```

```javascript
/**
 * Une ligne : ce qu'on sait de ce livrable, et les quatre gestes qu'on peut lui faire.
 *
 * Deux niveaux de remplissage, et c'est voulu (décision 1). Ce que le modèle retient —
 * identité, pages, gouttière, dos, état, chemins — se lit toujours, y compris à la
 * réouverture d'un projet fermé la veille. Ce que seule la composition a vu — dos rogné,
 * avertissements, polices de repli — ne paraît que dans la session qui a généré : le
 * `.ozalid` ne le retient pas, et l'inventer serait pire que de se taire.
 */
function ligneLivrable(d) {
  const ligne = h('div', undefined, 'livrable');
  ligne.id = `liv-${d.cle}`;
  const p = providers.find((pr) => pr.cle === d.gabarit);
  const pod = pods.find((x) => x.cle === d.pod);
  const dosPublie = pod?.papiers.find((pa) => pa.cle === d.papier)?.dos_publie ?? false;

  const infos = h('div', undefined, 'infos');
  infos.append(h('span', libelleDansGroupe(d), 'nom'));
  if (p) infos.append(h('p', noteFormat(p), 'note'));
  infos.append(noteMesure(d, dosPublie), noteEtat(d));

  // Les fichiers que la génération a écrits, reconstruits de la clé : leur nom ne dépend
  // que d'elle (`package::nom`), et le répertoire non plus. Les redemander au Rust
  // coûterait une commande pour une chaîne qu'on sait fabriquer.
  const r = packagesDeLaSession[d.cle];
  if (r?.package) {
    const q = r.package;
    const dl = h('dl');
    for (const [k, v] of [
      ['Pages', `${q.pages}${q.blanche ? ' (blanche de parité)' : ''}`],
      ['Papier', q.papier],
      ...(r.finition ? [['Finition', r.finition]] : []),
      ['Gouttière', `${nb(q.gouttiere, 1)} mm`],
      ['Dos', `${nb(q.dos)} mm`],
      ['Planche', `${nb(q.planche[0])} × ${nb(q.planche[1])} mm, FP ${nb(q.fond_perdu, 3)} mm`],
    ]) dl.append(h('dt', k), h('dd', v));
    infos.append(dl);
    // Le dos est composé sur une zone qui rogne ce qui dépasse, sans rien dire : un titre
    // coupé au pli ne se verrait qu'à l'impression.
    if (q.dos_requis !== null) {
      infos.append(h('p', `Dos de ${nb(q.dos)} mm pour un texte qui en réclame `
        + `${nb(q.dos_requis)} mm : il sera rogné au pli. Réduire le corps du dos, ou `
        + 'y éteindre un élément.', 'note alerte'));
    }
    // Une police que Typst a remplacée sans échouer : ce PDF-là part chez l'imprimeur.
    if (q.polices_introuvables.length) {
      infos.append(h('p', 'Police introuvable, composé dans une écriture de repli : '
        + `${q.polices_introuvables.join(', ')}. Le PDF ne suit pas la maquette.`,
      'note alerte'));
    }
    // En gris et non en rouge : les deux alertes ci-dessus disent qu'un PDF ne suit pas la
    // maquette, celles-ci qu'un tirage juste ne plaira peut-être pas. C'est un jugement
    // d'auteur, et le rouge perdrait son sens à couvrir les deux. Les phrases viennent du
    // Rust telles quelles : la fiche de téléversement les recopie, et un dossier relu trois
    // mois plus tard doit dire ce que l'écran disait.
    for (const a of q.avertissements) infos.append(h('p', a, 'note'));
    for (const c of cheminsGroupes(q.chemins)) infos.append(h('p', c, 'chemin'));
  }

  const img = h('img', undefined, 'vignette');
  img.id = `liv-vignette-${d.cle}`;
  img.alt = `Planche composée pour ${libelleDansGroupe(d)}`;
  // La source arrive de `chargerVignettes` ou du compte rendu de la session : la ligne se
  // monte sans attendre le disque.
  if (r?.vignette) img.src = r.vignette;

  ligne.append(infos, img, gestesLivrable(d));
  return ligne;
}
```

`gestesLivrable(d)` est la tâche 7. `noteMesure` existe (`livraison.js:284`) : lui retirer son
argument `perimees`, que l'état remplace ; `noteFormat` et `cheminsGroupes` ne bougent pas.

```javascript
/**
 * Les vignettes laissées sur le disque par les générations d'avant.
 *
 * Hors de la vue et après le montage des lignes : `livraison_vue` est rendue par toute
 * commande qui écrit, et un base64 par livrable à chaque frappe se paierait pour rien. La
 * ligne se monte sans, et la vignette s'y pose quand elle arrive — c'est le même parti que
 * l'aperçu de la Couverture.
 */
async function chargerVignettes() {
  const table = await invoke('livrable_vignettes');
  for (const [cle, donnee] of Object.entries(table)) {
    const img = $(`liv-vignette-${cle}`);
    if (img) img.src = donnee;
  }
}
```

Dans `src/styles.css`, `.groupe` (titre, espacement) et la reprise des règles de `.package`
(`:1384-1399`) à l'intérieur d'une ligne. **Corriger au passage le défaut connu** :
`.package dl dd:last-child { grid-column: 2 / -1; }` laisse le libellé « Planche » orphelin en
colonne 3 — c'est une dette relevée avant ce lot, et la règle déménage ici de toute façon.

- [ ] **Étape 4 : voir les tests passer**

```bash
node --test tests/*.test.js
```

Attendu : les sept passent. Les tests du compte rendu (`packages.test.js:846` à `:1072`)
rougissent sur leur sélecteur : `#packages` n'existe plus, leur contenu est dans
`liv-<clé>`. **Les reprendre un par un**, leur intention est intacte.

- [ ] **Étape 5 : commit**

```bash
git add src/livraison.js src/app.js src/styles.css tests/packages.test.js
git commit -m "Chaque livrable porte son compte rendu, chaque groupe porte son imprimeur"
```

---

### Tâche 7 : Les quatre boutons de la ligne

Modifier, Dupliquer, Régénérer, Supprimer. Les deux premiers passent par le formulaire de la
tâche 5 — Modifier en mode Remplacer, Dupliquer en mode Générer avec les axes recopiés ; les
deux derniers appellent directement leur verbe. `nettoyage_echoue` s'affiche ici, et nulle
part ailleurs : c'est un fait de la réponse à Remplacer, pas un état du livrable (verdict 4b).

**Fichiers :**
- Modifier : `src/livraison.js` (les boutons dans `ligneLivrable`)
- Modifier : `src/app.js` (rien à brancher — les écouteurs sont posés à la construction)
- Modifier : `tests/packages.test.js`

**Interfaces :**
- Consomme : `livrable_remplacer`, `livrable_regenerer`, `livrable_supprimer`, `armeSur`,
  `armerGeste`, `desarmerGeste`, `tente`, `lireFormulaire`, `ouvrirModification`.
- Produit : `function gestesLivrable(d)` — appelée par `ligneLivrable` (tâche 6) ;
  `function ouvrirModification(d)` ; `function ouvrirDuplication(d)` ;
  `function remplirFormulaire(d)` ; `async function pendantQueCaCompose(bt, mot, geste)` ;
  `function regenererLivrable(d)` ; `async function supprimerLivrable(d)` ;
  `let remplace` — la clé du livrable en cours de modification, ou `null`.
  Identifiants : `liv-modifier-<clé>`, `liv-dupliquer-<clé>`, `liv-regenerer-<clé>`,
  `liv-supprimer-<clé>`.

- [ ] **Étape 1 : écrire les tests qui échouent**

```javascript
/**
 * Modifier remplit le formulaire avec les axes de la ligne, **relevés compris**. Sans eux,
 * corriger un dos relevé deviendrait une ressaisie complète — et `livrable_regler` ayant
 * disparu, c'est le seul chemin qui reste (décision 2).
 */
test('Modifier reprend les axes et les relevés de la ligne', async () => {
  const { els } = await ouvre([POSTE], {}, {
    livrables: [{ ...chez(POSTE), papier: 'sans-formule', dos_mm: 17.4, fond_perdu_mm: 3 }],
  });
  els.get('liv-modifier-poste-110x170-broche-sans-formule')
    .dispatchEvent(new Evenement('click'));
  assert.strictEqual(els.get('inAjoutPapier').value, 'sans-formule');
  assert.strictEqual(els.get('inAjoutDos').value, '17.4');
  assert.strictEqual(els.get('inAjoutFp').value, '3');
});

/**
 * Le second verbe : en modification, le bouton remplace au lieu de générer, et il dit
 * laquelle des deux choses il va faire. Un bouton « Générer » qui remplacerait poserait un
 * doublon ou écraserait un package sans le dire.
 */
test('en modification, le formulaire remplace au lieu de générer', async () => {
  const { els, appels } = await ouvre([LULU]);
  els.get('liv-modifier-lulu-108x175-broche-standard')
    .dispatchEvent(new Evenement('click'));
  assert.match(els.get('btLivrableGenerer').textContent, /Remplacer/);
  els.get('btLivrableGenerer').dispatchEvent(new Evenement('click'));
  await pause();
  const [, args] = dernier(appels, 'livrable_remplacer');
  assert.strictEqual(args.cle, 'lulu-108x175-broche-standard');
  assert.strictEqual(dernier(appels, 'livrable_generer'), undefined);
});

/**
 * Dupliquer remplit le formulaire sans armer le remplacement : c'est un ajout, et le geste
 * pour lequel cet écran existe — comparer deux papiers d'un même livre — commence par là.
 */
test('Dupliquer prépare un ajout, pas un remplacement', async () => {
  const { els, appels } = await ouvre([LULU]);
  els.get('liv-dupliquer-lulu-108x175-broche-standard')
    .dispatchEvent(new Evenement('click'));
  assert.match(els.get('btLivrableGenerer').textContent, /Générer/);
  els.get('btLivrableGenerer').dispatchEvent(new Evenement('click'));
  await pause();
  assert.ok(dernier(appels, 'livrable_generer'), 'c\'est un ajout');
});

/**
 * Supprimer emporte la ligne, son package et les relevés qu'on y a saisis, sans reprise :
 * le premier clic arme, le second supprime. Même dispositif que le retrait d'avant le lot,
 * et que l'effacement d'une maquette, pour la même raison.
 */
test('Supprimer demande confirmation avant d\'emporter le package', async () => {
  const { els, appels } = await ouvre([LULU, KDP], {}, {
    livrables: [chez(LULU), chez(KDP)],
  });
  const bt = els.get('liv-supprimer-lulu-108x175-broche-standard');
  bt.dispatchEvent(new Evenement('click'));
  assert.strictEqual(dernier(appels, 'livrable_supprimer'), undefined, 'le premier clic arme');
  assert.match(bt.textContent, /Confirmer/);
  bt.dispatchEvent(new Evenement('click'));
  await pause();
  assert.strictEqual(dernier(appels, 'livrable_supprimer')[1].cle, 'lulu-108x175-broche-standard');
});

/**
 * Le dernier livrable ne se supprime pas : c'est lui qui donne le format sous lequel on
 * regarde la couverture. Le Rust refuse déjà, mais un bouton qui ne peut qu'échouer vaut
 * mieux éteint que refusé.
 */
test('le dernier livrable ne peut pas être supprimé', async () => {
  const { els } = await ouvre([LULU]);
  assert.strictEqual(els.get('liv-supprimer-lulu-108x175-broche-standard').disabled, true);
});

/**
 * Un nettoyage qui a échoué se dit, sinon un ancien répertoire qu'un remplacement n'a pas
 * su effacer survit en silence — et l'on retrouve six mois plus tard deux répertoires pour
 * un seul livrable, sans savoir lequel est parti chez l'imprimeur.
 */
test('un remplacement qui n\'a pas su nettoyer le dit', async () => {
  const { els } = await ouvre([LULU], {
    nettoyage_echoue: 'ancien répertoire non effacé : fichier verrouillé',
  });
  els.get('liv-modifier-lulu-108x175-broche-standard')
    .dispatchEvent(new Evenement('click'));
  els.get('btLivrableGenerer').dispatchEvent(new Evenement('click'));
  await pause();
  assert.match(els.get('etatLivraison').textContent, /non effacé/);
  assert.match(els.get('etatLivraison').className, /erreur|alerte/);
});
```

- [ ] **Étape 2 : voir les tests échouer**

```bash
node --test tests/packages.test.js
```

Attendu : les six échouent — aucun des quatre boutons n'existe.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `src/livraison.js`, les quatre boutons d'une ligne :

```javascript
/**
 * Les quatre verbes d'un livrable, dans l'ordre du geste : on modifie plus souvent qu'on
 * ne duplique, on duplique plus souvent qu'on ne régénère, et on supprime en dernier. Ce
 * qui défait est à droite, comme le retrait l'était.
 */
function gestesLivrable(d) {
  const bouton = (quoi, texte, ecoute) => {
    const b = h('button', texte);
    b.type = 'button';
    b.className = 'nu';
    b.id = `liv-${quoi}-${d.cle}`;
    b.addEventListener('click', ecoute);
    return b;
  };
  const gestes = h('div', undefined, 'gestes');
  // Supprimer emporte la ligne, son package et les relevés qu'on y a saisis, sans reprise,
  // au milieu de trois gestes qu'on fait couramment : le premier clic arme, le second
  // supprime. Même dispositif que le retrait d'avant le lot et que l'effacement d'une
  // maquette — et la raison s'est aggravée, puisque le geste emporte maintenant les
  // fichiers avec la ligne.
  const supprimer = bouton('supprimer', '⌫ Supprimer', () => {
    if (armeSur(supprimer)) {
      desarmerGeste();
      return supprimerLivrable(d);
    }
    armerGeste(supprimer, () => {
      supprimer.textContent = '⌫ Supprimer';
      supprimer.className = 'nu';
    });
    supprimer.textContent = 'Confirmer';
    supprimer.className = 'danger';
    return undefined;
  });
  // Le dernier ne se supprime pas : le Rust refuse, mais un bouton qui ne peut qu'échouer
  // vaut mieux éteint que refusé.
  supprimer.disabled = projet.livraison.livrables.length < 2;
  gestes.append(
    bouton('modifier', '✎ Modifier', () => ouvrirModification(d)),
    bouton('dupliquer', '⧉ Dupliquer', () => ouvrirDuplication(d)),
    bouton('regenerer', '⟳ Régénérer', () => regenererLivrable(d)),
    supprimer,
  );
  return gestes;
}
```

Le tout est le corps de `gestesLivrable(d)`, appelée par `ligneLivrable` (tâche 6) ; `bouton`
en est la fabrique locale. `armeSur`, `armerGeste` et `desarmerGeste` existent déjà et ne
changent pas.

```javascript
/**
 * Le livrable en cours de modification, ou `null` quand le formulaire ajoute.
 *
 * C'est ce qui donne au bouton son second verbe. Une modification abandonnée — on clique
 * Modifier puis on change d'avis — se défait en cliquant Dupliquer, ou en modifiant une
 * autre ligne : le formulaire n'a qu'un état, et il est toujours celui du dernier geste.
 */
let remplace = null;

function ouvrirModification(d) {
  remplace = d.cle;
  remplirFormulaire(d);
  $('btLivrableGenerer').textContent = 'Remplacer';
}

/** Duplique : les mêmes axes, mais c'est un ajout — le geste qui compare deux papiers. */
function ouvrirDuplication(d) {
  remplace = null;
  remplirFormulaire(d);
  $('btLivrableGenerer').textContent = 'Générer';
}
```

```javascript
/**
 * Remplit le formulaire avec les axes d'un livrable existant.
 *
 * L'ordre n'est pas négociable : le POD d'abord, puis `afficherAxesDuPod`, **puis** les
 * quatre autres valeurs — les listes de format, reliure, papier et pelliculage n'existent
 * qu'après, et poser une valeur dans une liste vide la perd sans rien dire.
 *
 * Les deux relevés sont repris, et c'est la raison d'être de Modifier : depuis que
 * `livrable_regler` a disparu, c'est le seul chemin par lequel un dos relevé se corrige
 * (décision 2). Les oublier ferait de la correction d'un chiffre une ressaisie complète.
 */
function remplirFormulaire(d) {
  $('inAjoutPod').value = d.pod;
  afficherAxesDuPod();
  $('inAjoutFormat').value = d.format;
  $('inAjoutReliure').value = d.reliure;
  $('inAjoutPapier').value = d.papier;
  if ($('inAjoutFinition')) $('inAjoutFinition').value = d.finition ?? '';
  // Les relevés dépendent du papier, qui vient d'être posé : les champs n'existent qu'une
  // fois `afficherRelevesDuFormulaire` rappelée avec le bon papier.
  afficherRelevesDuFormulaire(pods.find((x) => x.cle === d.pod));
  if ($('inAjoutDos')) $('inAjoutDos').value = d.dos_mm ?? '';
  if ($('inAjoutFp')) $('inAjoutFp').value = d.fond_perdu_mm ?? '';
}
```

`genererLivrable` (tâche 5) se dédouble selon `remplace` :

```javascript
    const r = remplace === null
      ? await invoke('livrable_generer', { livrable: lireFormulaire() })
      : await invoke('livrable_remplacer', { cle: remplace, livrable: lireFormulaire() });
    remplace = null;
    $('btLivrableGenerer').textContent = 'Générer';
    afficherProjet(r.projet);
    retenirPackagesDeLaSession(r.packages);
    // Ce que l'effacement de l'ancien répertoire n'a pas pu faire : la composition a réussi
    // et le projet porte le livrable neuf, mais un répertoire est resté. Sans ce mot, il
    // survit en silence — et l'on retrouve deux répertoires pour un livrable, sans savoir
    // lequel est parti chez l'imprimeur.
    if (r.nettoyage_echoue) {
      $('etatLivraison').textContent = r.nettoyage_echoue;
      $('etatLivraison').className = 'etat erreur';
    } else {
      $('etatLivraison').textContent = '';
    }
```

```javascript
/**
 * L'attente d'un verbe qui compose : bouton éteint, ligne d'état, et le projet remis à
 * l'écran quoi qu'il arrive. Trois verbes la partagent — un dispositif recopié trois fois
 * finit par diverger sur le seul qui compte, celui qui échoue.
 */
async function pendantQueCaCompose(bt, mot, geste) {
  bt.disabled = true;
  $('etatLivraison').className = 'etat';
  $('etatLivraison').textContent = mot;
  try {
    const r = await geste();
    afficherProjet(r.projet);
    retenirPackagesDeLaSession(r.packages);
    $('etatLivraison').textContent = '';
    return r;
  } catch (e) {
    $('etatLivraison').textContent = String(e);
    $('etatLivraison').className = 'etat erreur';
    return null;
  } finally {
    bt.disabled = false;
  }
}

/**
 * Régénérer : recompose sans toucher aux axes.
 *
 * Peut légitimement **copier** l'intérieur d'un livrable du même gabarit déjà à jour au
 * lieu de le recomposer (spec § 4) : ce n'est pas un raté, c'est ce qui rend la
 * comparaison de deux papiers gratuite. Seul « Tout regénérer » recompose toujours.
 */
function regenererLivrable(d) {
  return pendantQueCaCompose(
    $(`liv-regenerer-${d.cle}`),
    'recomposition du package…',
    () => invoke('livrable_regenerer', { cle: d.cle })
  );
}

/**
 * Supprimer : efface les fichiers connus, retire le répertoire s'il est vide, retire le
 * livrable.
 *
 * Ce qui restait et que l'application n'a pas écrit **survit et se nomme** : le répertoire
 * reste pour lui. Le taire laisserait croire à un effacement complet, et l'on chercherait
 * six mois plus tard pourquoi un répertoire d'un livrable disparu traîne encore.
 */
async function supprimerLivrable(d) {
  await tente(async () => {
    const r = await invoke('livrable_supprimer', { cle: d.cle });
    afficherProjet(r.projet);
    if (r.nettoyage.etrangers.length) {
      $('etatLivraison').className = 'etat';
      $('etatLivraison').textContent = 'Le répertoire survit pour ce que l\'application '
        + `n'a pas écrit : ${r.nettoyage.etrangers.join(', ')}.`;
    }
  });
}
```

`genererLivrable` (tâche 5) se réécrit sur `pendantQueCaCompose` : le corps montré à la
tâche 5 en était la première version, avec un seul appelant.

- [ ] **Étape 4 : voir les tests passer**

```bash
node --test tests/*.test.js
```

Attendu : les six passent.

- [ ] **Étape 5 : commit**

```bash
git add src/livraison.js tests/packages.test.js
git commit -m "Chaque ligne porte ses quatre verbes, et dit ce qu'un nettoyage n'a pas pu faire"
```

---
### Tâche 8 : « Tout regénérer » en tête d'étape, et la zone intermédiaire qui disparaît

`packager` ne change pas de principe — c'est « Tout regénérer » (spec § 6). Ce qui change est
sa place et ce qu'il fait de sa réponse : le bouton monte en tête d'étape, et les packages
qu'il rend entrent dans les lignes au lieu d'un bloc à part. `#packages` disparaît du
balisage, et avec lui `afficherPackages`.

Un bouton par groupe serait un troisième verbe à expliquer : « Tout regénérer » reste
**global**.

**Fichiers :**
- Modifier : `src/index.html` (`#packages` retiré, `btPackager` déplacé et renommé)
- Modifier : `src/livraison.js` (`packager` rend ses résultats aux lignes)
- Modifier : `src/app.js` (le branchement `:1444`)
- Modifier : `tests/packages.test.js`

**Interfaces :**
- Consomme : `packager`, `retenirPackagesDeLaSession` (tâche 6).
- Produit : `#btToutRegenerer` remplace `#btPackager` ; `#etatPackages` devient
  `#etatLivraison`, partagé avec le formulaire ; `#packages` n'existe plus.

- [ ] **Étape 1 : écrire les tests qui échouent**

```javascript
/**
 * « Tout regénérer » recompose tout et rend ses comptes rendus **aux lignes** : il n'y a
 * plus de zone intermédiaire où les lire. C'est ce qui fait qu'une ligne dit la même chose
 * qu'on vienne de la générer seule ou avec les autres.
 */
test('Tout regénérer verse ses comptes rendus dans les lignes', async () => {
  const { els } = await ouvre([LULU], { packager: [PAQUET] });
  els.get('btToutRegenerer').dispatchEvent(new Evenement('click'));
  await pause();
  assert.strictEqual(els.get('packages'), undefined, 'plus de zone intermédiaire');
  assert.match(
    els.get('liv-lulu-108x175-broche-standard').textContent, /262 pages/
  );
});

/**
 * L'attente garde son dispositif : bouton éteint et ligne d'état. Le temps de composition
 * ne disparaît pas, il se répartit — et un bouton qui reste cliquable pendant qu'il
 * compose invite à lancer deux compositions concurrentes.
 */
test('Tout regénérer éteint son bouton et dit qu\'il travaille', async () => {
  let relache;
  const { els } = await ouvre([LULU], {
    packager: () => new Promise((r) => { relache = () => r([PAQUET]); }),
  });
  els.get('btToutRegenerer').dispatchEvent(new Evenement('click'));
  await pause();
  assert.strictEqual(els.get('btToutRegenerer').disabled, true);
  assert.match(els.get('etatLivraison').textContent, /composition/);
  relache();
  await pause();
  assert.strictEqual(els.get('btToutRegenerer').disabled, false);
  assert.strictEqual(els.get('etatLivraison').textContent, '');
});
```

- [ ] **Étape 2 : voir les tests échouer**

```bash
node --test tests/packages.test.js
```

Attendu : les deux échouent — `btToutRegenerer` n'existe pas, `packages` existe encore.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `src/index.html`, la ligne du bouton (`:391-395`) devient, **au-dessus** de `#livrables`
et sous le formulaire :

```html
      <div class="ligne">
        <button id="btToutRegenerer" type="button">Tout regénérer</button>
      </div>
```

et `<div id="packages" class="resultat" hidden></div>` est **supprimé**. Le bouton n'est plus
`primaire` : le noir de l'étape appartient désormais à « Générer », qui est le geste courant ;
« Tout regénérer » est le geste de rattrapage.

Dans `src/livraison.js`, `packager` verse dans les lignes au lieu d'un bloc, et
`afficherPackages` disparaît — son corps a déménagé dans `ligneLivrable` à la tâche 6 :

```javascript
/**
 * Tout regénérer : recompose chaque livrable, et rend les comptes rendus aux lignes.
 *
 * Global, et sans bouton par groupe : ce serait un troisième verbe à expliquer, entre le
 * « Régénérer » d'une ligne et celui-ci.
 */
async function packager() {
  const combien = projet.livraison.livrables.length;
  const bt = $('btToutRegenerer');
  bt.disabled = true;
  $('etatLivraison').className = 'etat';
  $('etatLivraison').textContent = `composition de ${combien} package(s)…`;
  try {
    // Générer compose : le projet revient mesuré, et le pied le relit là où il est
    // enregistré — sans quoi il dirait « dos non composé » sous une ligne qui vient de
    // donner le dos.
    const r = await invoke('packager');
    afficherProjet(r.projet);
    retenirPackagesDeLaSession(r.packages);
    $('etatLivraison').textContent = '';
  } catch (e) {
    $('etatLivraison').textContent = String(e);
    $('etatLivraison').className = 'etat erreur';
  } finally {
    bt.disabled = false;
  }
}
```

Dans `src/app.js`, `$('btPackager')` devient `$('btToutRegenerer')` (`:1444`), et le
`$('packages').replaceChildren()` de la fermeture de projet (`app.js:910` et alentours) est
retiré : la liste des livrables suffit, il n'y a plus de zone séparée à vider. **Chercher
toutes les occurrences de `'packages'` et `'etatPackages'` dans `src/`** avant de commiter :
un `$()` sur un identifiant disparu rend `undefined` et casse au premier accès.

- [ ] **Étape 4 : voir les tests passer**

```bash
node --test tests/*.test.js
grep -rn "btPackager\|etatPackages\|'packages'" src/
```

Attendu : les deux passent, et le `grep` ne rend rien.

- [ ] **Étape 5 : commit**

```bash
git add src/index.html src/livraison.js src/app.js tests/packages.test.js
git commit -m "Tout regénérer passe en tête d'étape, et la zone intermédiaire disparaît"
```

---

### Tâche 9 : Les trois commandes dont les gestes n'existent plus

`livrable_ajouter`, `livrable_regler` et `livrable_retirer` disparaissent (spec § 6). En
dernier, quand plus rien ne les appelle : les supprimer plus tôt aurait cassé l'écran pendant
cinq tâches. Le garde de contrat `contrats.test.js:253` — « chaque commande appelée par le
front est déclarée au Rust » — est ce qui prouve qu'aucun appel n'a été oublié.

Avec elles part `reglage_refuse`, dont le second membre — une finition que le POD ne porte pas
se refuse — doit **survivre** : `livrable_remplacer` le tient déjà depuis le lot 2
(reconnaissance du lot 2, verdict 1c). Le vérifier avant de supprimer, pas après.

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs` (les trois commandes et ce qu'elles seules employaient)
- Modifier : `src-tauri/src/lib.rs` (`invoke_handler`, `:130-132`)
- Modifier : `tests/coquille.test.js`, `tests/packages.test.js` (les faux perdent trois `case`)

**Interfaces :**
- Consomme : rien.
- Produit : rien — cette tâche retire.

- [ ] **Étape 1 : écrire le test qui échoue**

```javascript
/**
 * Les trois commandes dont les gestes n'existent plus ne sont plus appelées de nulle part.
 * Ce test est le garde de la suppression : il rougit si un écouteur oublié les rappelle,
 * là où le garde de contrat ne verrait qu'une commande non déclarée.
 */
test('le front n\'appelle plus les commandes du geste en deux temps', () => {
  const source = ['app.js', 'livraison.js', 'couverture.js', 'envois.js']
    .map((f) => fs.readFileSync(path.join(__dirname, '..', 'src', f), 'utf8'))
    .join('\n');
  for (const cmd of ['livrable_ajouter', 'livrable_regler', 'livrable_retirer']) {
    assert.doesNotMatch(source, new RegExp(cmd), `${cmd} est encore appelée`);
  }
});
```

À poser dans `tests/contrats.test.js`, à côté du garde `:253` : c'est le même genre de
vérification — lire les vrais fichiers des deux côtés —, et c'est là qu'on la cherchera.

- [ ] **Étape 2 : voir le test échouer**

```bash
node --test tests/contrats.test.js
```

Attendu : il passe **déjà** si les tâches 5 à 8 ont fait leur travail. Dans ce cas, le voir
échouer demande une mutation ciblée : remettre un `invoke('livrable_regler', …)` dans
`livraison.js`, relancer, constater le rouge, l'enlever. **Un test qui n'a jamais été rouge ne
protège rien** — c'est la règle du dépôt, et elle vaut aussi pour un test d'absence.

- [ ] **Étape 3 : écrire l'implémentation**

Supprimer dans `commands.rs` les trois `#[tauri::command]` et, **après avoir vérifié qu'elles
n'ont pas d'autre appelant**, ce qu'elles seules employaient — `reglage_refuse` en particulier,
dont il faut d'abord confirmer que `livrable_remplacer` ne s'en sert pas. Retirer les trois
lignes de `lib.rs:130-132`. Retirer les trois `case` du faux de `coquille.test.js` et les `if`
correspondants de `packages.test.js`.

```bash
cd src-tauri && grep -rn "livrable_ajouter\|livrable_regler\|livrable_retirer\|reglage_refuse" src/
```

Ce que le `grep` rend encore après la suppression doit être **lu**, pas supprimé au jugé : un
test Rust qui éprouvait `reglage_refuse` éprouvait peut-être une règle que `livrable_remplacer`
tient encore.

- [ ] **Étape 4 : voir les tests passer**

```bash
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
cd .. && node --test tests/*.test.js
```

Attendu : tout vert. `clippy` doit être **muet sur les fonctions mortes** : un
`dead_code` signale une fonction que la suppression a laissée orpheline, et c'est exactement
ce qu'on veut voir.

- [ ] **Étape 5 : le témoin, puis commit**

```bash
cd src-tauri && cargo run --example temoin      # attendu : 98 / 118 / 100
git add src-tauri/src/commands.rs src-tauri/src/lib.rs tests/
git commit -m "Les trois commandes du geste en deux temps disparaissent"
```

---

### Tâche 10 : Le README dit le geste neuf

La section « 3 · Livraison » (`README.md:165`) décrit le geste en deux temps — déclarer, puis
« Générer les packages ». Elle est fausse dès la tâche 8. Le § 7 de la spec la range dans « ce
qui bouge ailleurs ».

**Fichiers :**
- Modifier : `README.md` (section « 3 · Livraison »)

**Interfaces :** aucune.

- [ ] **Étape 1 : relire l'écran, puis écrire**

Pas de test à écrire : c'est de la prose. Mais elle se vérifie — **lancer l'application et
écrire ce qu'on voit**, pas ce que le plan annonçait. L'ordre compte : cette tâche vient après
le « À l'œil » ci-dessous, jamais avant.

Ce qui doit changer dans le texte :
- Le geste : un livrable se **génère** d'un coup, depuis le formulaire ; il n'y a plus de
  liste à remplir puis un bouton à cliquer.
- Les quatre verbes de la ligne, et ce que chacun emporte — Supprimer efface les fichiers
  connus, conserve un fichier étranger et le nomme.
- « Tout regénérer » : ce qu'il fait de plus que « Régénérer » d'une ligne, à savoir
  recomposer toujours, là où Régénérer peut légitimement copier l'intérieur d'un livrable du
  même gabarit déjà à jour.
- Le marquage de péremption : ce que « le texte a changé depuis cette génération » veut dire,
  et pourquoi il distingue le texte de la couverture.
- Les relevés : ils sont **dans le formulaire**, et se corrigent par Modifier.

Ce qui ne change pas et ne doit pas être réécrit : le grisé de la reliure non outillée et sa
réserve en « Limites connues », le paragraphe des ebooks, la phrase sur la vignette qui répond
à « est-ce que ça tient ? ».

- [ ] **Étape 2 : commit**

```bash
git add README.md
git commit -m "Le README décrit le geste d'un seul temps"
```

---

## À l'œil, avant de clore le lot

Ce que le faux DOM ne peut pas dire (spec § 9). À faire **sur le projet réel**, aux deux
largeurs de fenêtre, avant la tâche 10 :

```bash
touch src-tauri/src/lib.rs && cargo build      # le front est embarqué à la compilation
cargo tauri dev
```

1. **La liste tient dans la fenêtre.** Trois livrables chez deux imprimeurs, chacun avec sa
   vignette et son compte rendu : est-ce qu'on voit encore le formulaire sans faire défiler ?
2. **Le groupe se lit comme un groupe.** L'imprimeur en tête doit se distinguer des lignes
   sans crier — c'est un titre, pas une alerte.
3. **Le marquage de péremption se voit sans être criard.** Modifier le manuscrit, revenir :
   les lignes doivent dire « le texte a changé » sans que la page devienne rouge.
4. **La vignette est là à la réouverture.** Générer, fermer le projet, rouvrir : les vignettes
   doivent revenir. C'est le seul point que les tests ne peuvent pas prouver, puisqu'ils ne
   lisent pas de disque.
5. **Le pied n'a pas bougé.** Il liste les livrables, suit le visé, et son libellé porte
   toujours l'imprimeur — c'est là qu'il doit le porter, aucun groupe ne le dit pour lui.
6. **Le point que le lot 2 n'a pas fait** : ouvrir un `.ozalid` enregistré après génération
   et y lire la sous-table `[livraison.livrables.generation]`. C'est une dette du lot
   précédent, et ce lot est la première occasion de la solder.

## Ce que ce lot ne fait pas

- **Il ne touche pas au catalogue, aux formules de dos ni aux relevés d'imprimeur** (spec,
  hors périmètre).
- **Il ne change pas le format du `.ozalid`** : décision 1. Une ligne rouverte ne dira jamais
  ce qu'un dos rogné avait signalé la veille — c'est assumé, et c'est le prix de ne pas
  rouvrir le modèle.
- **Il ne compose pas une reliure que l'application n'outille pas.** Le grisé reste, sa
  réserve est au README.
- **Il ne touche pas aux ebooks ni aux envois**, qui gardent leurs boutons et leurs étapes.
- **Il ne reprend pas les pistes de mise en page** rangées hors périmètre jusqu'après ce lot :
  la base de 12 rem de `.livrable .nom`, le paragraphe d'en-tête sur deux lignes. Le
  groupement change la donne pour les deux — les rouvrir après, avec l'écran neuf sous les
  yeux. Seul le défaut du `dd:last-child` est corrigé, parce que la règle déménage de toute
  façon (tâche 6).
- **Il ne gèle pas le relevé TheBookEdition par un test**, contrairement à BoD et Lulu : c'est
  une dette antérieure, sans rapport avec l'écran.
