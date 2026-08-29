# Livraison refondue, lot 1 — le modèle

> **Pour un exécutant agentique :** SOUS-COMPÉTENCE REQUISE : `superpowers:subagent-driven-development`
> (recommandé) ou `superpowers:executing-plans` pour exécuter ce plan tâche par tâche. Les
> étapes sont des cases à cocher (`- [x]`).

**But :** un livrable retient ce que sa génération a produit — deux empreintes, ou un message
d'échec — et le projet sait dire, sans rien composer, si son intérieur ou sa couverture a
bougé depuis. Aucun changement visible à l'écran : tout se vérifie en tests Rust.

**Architecture :** un module neuf `empreinte.rs`, court et mono-sujet, à la manière de
`detourage.rs` et `police.rs`. Il porte un condensé déterministe (FNV-1a) — **et non celui de
`commands.rs`**, qui repose sur `DefaultHasher` et n'est pas stable d'une version de Rust à
l'autre (reconnaissance 2b) — puis les deux empreintes du couple projet × livrable, et la
comparaison qui en découle. `Livrable` gagne un cinquième champ, optionnel, en **dernière
position** de la structure. `VERSION` ne bouge pas : c'est le parti déjà pris pour `livraison`
et `envois`.

**Pile :** Rust 2021, `serde` + `serde_json` (déjà en dépendance) + `toml 0.8`. Tests :
`cargo test` depuis `src-tauri/`, `cargo run --example temoin` comme témoin.

**Spec :** `docs/superpowers/specs/2026-08-29-livraison-refondue-design.md` (§ 2).
**Reconnaissance :** `docs/superpowers/2026-08-29-reconnaissance-livraison-lot-1.md` — les
verdicts cités ici (1a à 6a) y sont, chacun appuyé sur un fichier et une ligne.

---

## Décisions arbitrées (utilisateur, 29/08) — ne pas les rouvrir

1. **Deux empreintes, pas une** : intérieur et couverture séparément. Elles servent deux
   choses à la fois — dire *quoi* a bougé, et permettre à un intérieur d'être réutilisé alors
   que la couverture a changé (lot 2).
2. **La péremption couvre la couverture**, que le mécanisme actuel ignore.
3. **L'état est optionnel dans le fichier** : un `.ozalid` d'avant s'ouvre en *jamais généré*.

## Contraintes globales

- **Français** dans les commentaires, les messages et les commits ; termes techniques
  anglais conservés tels quels.
- **Aucun test neuf ne compte s'il n'a pas été vu échouer.** TDD strict, ou mutation ciblée.
- `VERSION` du `.ozalid` **ne change pas**.
- Le témoin doit valoir le même compte de pages qu'avant le lot.

## Avant chaque commit

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
cd src-tauri && cargo test
cd .. && node --test tests/*.test.js
cd src-tauri && cargo run --example temoin     # dès qu'un fichier de src-tauri/ a bougé
```

`clippy` est rouge sur la baseline depuis rustc 1.98 — `police.rs:123` et
`examples/packager.rs:32`, lint `chunks_exact_to_as_chunks`. Ce sont les deux seuls avertis-
sements admis ; tout autre est de votre fait.

## Structure des fichiers

| fichier | rôle |
|---|---|
| `src-tauri/src/empreinte.rs` | **créé** — le condensé, les deux empreintes, la comparaison |
| `src-tauri/src/lib.rs` | **modifié** — `pub mod empreinte;` entre `ebook` et `envoi` |
| `src-tauri/src/projet.rs` | **modifié** — `Generation` et le champ de `Livrable` |

---

### Tâche 1 : Un condensé qui ne bouge pas entre deux versions du binaire

**Fichiers :**
- Créer : `src-tauri/src/empreinte.rs`
- Modifier : `src-tauri/src/lib.rs` (une ligne)

**Interfaces :**
- Consomme : rien.
- Produit : `pub fn condense(octets: &[u8]) -> String` — seize caractères hexadécimaux.

- [ ] **Étape 1 : écrire le test qui échoue**

Dans `src-tauri/src/empreinte.rs`, sous `#[cfg(test)] mod tests` :

```rust
/// Les valeurs sont **gelées**, et c'est tout l'intérêt du test : cette empreinte est
/// écrite dans le `.ozalid` et relue par un binaire qu'on aura recompilé entre-temps. Un
/// algorithme qu'on changerait sans y penser marquerait d'un coup tous les packages de
/// tous les projets comme périmés, sans que rien à l'écran puisse l'expliquer. Les trois
/// vecteurs sont ceux de la spécification FNV-1a 64 bits.
#[test]
fn le_condense_est_gele() {
    assert_eq!(condense(b""), "cbf29ce484222325");
    assert_eq!(condense(b"a"), "af63dc4c8601ec8c");
    assert_eq!(condense(b"ozalid"), "dc0fb47ed8d84474");
}

/// Deux entrées voisines ne se confondent pas : sans quoi changer une lettre du titre
/// laisserait la couverture marquée à jour.
#[test]
fn deux_entrees_voisines_ne_se_condensent_pas_pareil() {
    assert_ne!(condense(b"a"), condense(b"b"));
}
```

- [ ] **Étape 2 : voir le test échouer**

Ajouter `pub mod empreinte;` dans `src-tauri/src/lib.rs`, entre `pub mod ebook;` et
`pub mod envoi;` — la liste est alphabétique.

Run : `cd src-tauri && cargo test condense`
Attendu : ÉCHEC à la compilation, `cannot find function 'condense'`.

- [ ] **Étape 3 : écrire l'implémentation**

En tête de `src-tauri/src/empreinte.rs` :

```rust
//! Ce qui dit qu'un package n'est plus celui du projet qu'on a sous les yeux.
//!
//! Deux empreintes par livrable — l'intérieur, la couverture — parce qu'elles répondent à
//! deux questions différentes : laquelle des deux moitiés a bougé, et un intérieur déjà
//! composé peut-il resservir alors que la couverture, elle, a changé.

/// Un condensé FNV-1a 64 bits, en seize caractères hexadécimaux.
///
/// **Écrit ici et non repris de `commands::empreinte`**, qui repose sur `DefaultHasher`.
/// Celle-là nomme un répertoire de rendus : une valeur qui change fabrique un répertoire
/// neuf et l'on recalcule, personne ne le voit. Celle-ci est écrite dans le `.ozalid` et
/// relue par un binaire recompilé — or la bibliothèque standard ne garantit pas que
/// `DefaultHasher` rende la même valeur d'une version de Rust à l'autre. Une mise à jour
/// de l'application marquerait alors tous les packages périmés d'un coup, sans que rien
/// ne l'explique. FNV-1a, lui, est une spécification : il ne bougera jamais.
pub fn condense(octets: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for o in octets {
        h ^= u64::from(*o);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}
```

- [ ] **Étape 4 : voir les tests passer**

Run : `cd src-tauri && cargo test condense`
Attendu : 2 passés.

- [ ] **Étape 5 : commit**

```bash
cd src-tauri && cargo fmt && cd ..
git add src-tauri/src/empreinte.rs src-tauri/src/lib.rs
git commit -m "Une empreinte qui survit à la recompilation du binaire"
```

---

### Tâche 2 : Les deux empreintes du couple projet × livrable

**Fichiers :**
- Modifier : `src-tauri/src/empreinte.rs`
- Test : même fichier, `mod tests`

**Interfaces :**
- Consomme : `condense` (tâche 1) ; `Projet` et `Livrable` (`projet.rs`) ;
  `catalogue::resout` et `Resolu::empreinte` (`catalogue.rs:1079`) ;
  `Livraison::mesure(&str) -> Option<&Mesure>`.
- Produit :
  - `pub fn interieur(projet: &Projet, l: &Livrable) -> String`
  - `pub fn couverture(projet: &Projet, l: &Livrable) -> String`

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
/// Ce qui compose l'intérieur le périme, et rien d'autre. Le manuscrit fait la
/// pagination ; l'identité du livre fait la page de titre et les liminaires ; les
/// réglages font la police et les corps ; le gabarit fait la boîte.
#[test]
fn l_empreinte_d_interieur_suit_ce_qui_compose_l_interieur() {
    let p = projet_d_essai();
    let l = p.meta.livraison.livrables[0].clone();
    let depart = interieur(&p, &l);

    let mut q = p.clone();
    q.texte.push_str("\n\nUn paragraphe de plus.");
    assert_ne!(interieur(&q, &l), depart, "le manuscrit doit périmer");

    let mut q = p.clone();
    q.meta.livre.titre = "Un autre titre".into();
    assert_ne!(interieur(&q, &l), depart, "le titre doit périmer");

    let mut q = p.clone();
    q.meta.interieur.corps = 11.0;
    assert_ne!(interieur(&q, &l), depart, "le corps doit périmer");
}

/// Et ce qui ne le compose pas ne le périme pas : la couverture retouchée ou un envoi
/// ajouté ne changent pas un octet de l'intérieur. Sans ce bord, la liste crierait au
/// loup et on cesserait de la lire.
#[test]
fn l_empreinte_d_interieur_ignore_la_couverture_et_les_envois() {
    let p = projet_d_essai();
    let l = p.meta.livraison.livrables[0].clone();
    let depart = interieur(&p, &l);

    let mut q = p.clone();
    q.meta.couverture.maquette = None;
    assert_eq!(interieur(&q, &l), depart, "la couverture ne périme pas l'intérieur");

    let mut q = p.clone();
    q.meta.envois.liste.push(crate::envoi::Envoi::default());
    assert_eq!(interieur(&q, &l), depart, "un envoi ne périme pas l'intérieur");
}

/// La couverture porte le dos, donc le papier et la pagination : un changement de police
/// repagine, le dos bouge, et la planche déjà écrite est fausse. Sans ces deux morceaux,
/// elle se dirait à jour — c'est le risque nommé au § 8 de la spec.
#[test]
fn l_empreinte_de_couverture_suit_le_dos() {
    let p = projet_d_essai();
    let l = p.meta.livraison.livrables[0].clone();
    let depart = couverture(&p, &l);

    let mut autre_papier = l.clone();
    autre_papier.fabrication.papier = "blanc-90".into();
    assert_ne!(couverture(&p, &autre_papier), depart, "le papier fait le dos");

    let mut q = p.clone();
    q.meta.livraison.retenir_mesure(
        &l.fabrication.cle_gabarit(),
        crate::projet::Mesure { pages: 400, ..mesure_d_essai() },
    );
    assert_ne!(couverture(&q, &l), depart, "la pagination fait le dos");

    let mut q = p.clone();
    q.meta.couverture.maquette = None;
    assert_ne!(couverture(&q, &l), depart, "la maquette fait la planche");

    let mut q = p.clone();
    q.images.insert("premiere.jpg".into(), vec![1, 2, 3]);
    assert_ne!(couverture(&q, &l), depart, "l'image fait la planche");
}

/// Le manuscrit ne touche pas la planche : c'est ce qui permet, au lot 2, de recomposer
/// une couverture sans recomposer l'intérieur — et l'inverse.
#[test]
fn l_empreinte_de_couverture_ignore_le_manuscrit() {
    let p = projet_d_essai();
    let l = p.meta.livraison.livrables[0].clone();
    let depart = couverture(&p, &l);
    let mut q = p.clone();
    q.texte.push_str("\n\nUn paragraphe de plus.");
    assert_eq!(
        couverture(&q, &l),
        depart,
        "le manuscrit ne périme la couverture que par la pagination, qui est retenue à part"
    );
}
```

Les deux fabriques d'essai, dans le même `mod tests` :

```rust
/// `Livre` ne dérive pas `Default` : ses quatorze champs se posent un à un, comme
/// `package::tests::livre_d_essai` et `projet::tests::livre` le font déjà.
fn livre_d_essai() -> crate::projet::Livre {
    crate::projet::Livre {
        isbn: String::new(),
        depot_legal: String::new(),
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

fn projet_d_essai() -> crate::projet::Projet {
    let mut p = crate::projet::Projet::nouveau(
        livre_d_essai(),
        "## 01 - Un\n\nParagraphe.".into(),
    );
    p.meta.couverture.maquette = Some(
        crate::maquettes::par_cle(None, "filets")
            .expect("maquette fournie « filets »")
            .couverture,
    );
    let cle = p.meta.livraison.livrables[0].fabrication.cle_gabarit();
    p.meta.livraison.retenir_mesure(&cle, mesure_d_essai());
    p
}

fn mesure_d_essai() -> crate::projet::Mesure {
    crate::projet::Mesure {
        pages: 98,
        gouttiere: 14.0,
        blanche: false,
        empreinte: None,
        polices_introuvables: Vec::new(),
    }
}
```

- [ ] **Étape 2 : voir les tests échouer**

Run : `cd src-tauri && cargo test empreinte::tests`
Attendu : ÉCHEC à la compilation, `cannot find function 'interieur'`.

- [ ] **Étape 3 : écrire l'implémentation**

```rust
use crate::projet::{Livrable, Projet};

/// L'empreinte de ce qui compose l'**intérieur** de ce livrable.
///
/// Le gabarit y entre par `Resolu::empreinte` — format, marges, gouttières —, la même
/// valeur que `Mesure::empreinte` retient déjà depuis le lot 2 du catalogue. Un gabarit
/// que le catalogue ne porte plus rend une empreinte vide : le livrable paraîtra périmé,
/// ce qui est vrai, et `normalise` l'élaguera à la prochaine ouverture.
pub fn interieur(projet: &Projet, l: &Livrable) -> String {
    let gabarit = crate::catalogue::resout(&l.fabrication)
        .map(|r| r.empreinte())
        .unwrap_or_default();
    condense(
        [
            condense(projet.texte.as_bytes()),
            json(&projet.meta.livre),
            json(&projet.meta.manuscrit),
            json(&projet.meta.interieur),
            gabarit,
        ]
        .join("|")
        .as_bytes(),
    )
}

/// L'empreinte de ce qui compose la **planche** de ce livrable.
///
/// Le livre y figure comme dans l'autre : la couverture cite `%TITRE%` et `%AUTEUR%`, et
/// l'oublier ici laisserait une moitié du livre à jour et l'autre fausse.
///
/// Le papier et la pagination y figurent parce que le **dos** en découle. On y met les
/// deux plutôt que le dos calculé : le dos est une fonction pure de ces deux-là, et le
/// calculer ici obligerait ce module à connaître `planche`, qu'il n'a aucune raison de
/// connaître.
pub fn couverture(projet: &Projet, l: &Livrable) -> String {
    let images: Vec<String> = projet
        .images
        .iter()
        .map(|(nom, octets)| format!("{nom}:{}", condense(octets)))
        .collect();
    let pages = projet
        .meta
        .livraison
        .mesure(&l.fabrication.cle_gabarit())
        .map(|m| m.pages.to_string())
        .unwrap_or_default();
    condense(
        [
            json(&projet.meta.livre),
            json(&projet.meta.couverture),
            images.join(","),
            l.fabrication.papier.clone(),
            l.dos_mm.map(|d| d.to_string()).unwrap_or_default(),
            pages,
        ]
        .join("|")
        .as_bytes(),
    )
}

/// La forme sérialisée d'un morceau de métadonnées, pour le condenser.
///
/// `serde_json` et non `toml` : TOML exige que les valeurs précèdent les tables, et
/// refuse certaines structures que l'on veut seulement décrire. Le JSON n'a pas cette
/// contrainte, et il est déjà en dépendance.
///
/// Une erreur devient un morceau au lieu d'une panique : cette fonction est appelée à
/// chaque vue, et faire tomber l'application pour un condensé serait hors de proportion.
/// Rendre une chaîne vide serait pire — le morceau cesserait silencieusement de compter.
fn json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|e| format!("!{e}"))
}
```

- [ ] **Étape 4 : voir les tests passer**

Run : `cd src-tauri && cargo test empreinte::tests`
Attendu : 5 passés.

- [ ] **Étape 5 : commit**

```bash
cd src-tauri && cargo fmt && cargo clippy --all-targets -- -D warnings ; cd ..
git add src-tauri/src/empreinte.rs
git commit -m "Deux empreintes par livrable : ce qui compose l'intérieur, ce qui compose la planche"
```

---

### Tâche 3 : Le livrable retient ce que sa génération a laissé

**Fichiers :**
- Modifier : `src-tauri/src/projet.rs` (structure `Livrable`, vers la ligne 300)
- Test : `src-tauri/src/projet.rs`, `mod tests`

**Interfaces :**
- Consomme : rien des tâches précédentes.
- Produit : `pub enum Generation { Jamais, Fait { interieur: String, couverture: String }, Echec { message: String } }`
  et le champ `pub generation: Generation` sur `Livrable`.

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
/// Un `.ozalid` écrit avant ce lot s'ouvre sans un mot, ses livrables en *jamais
/// généré*, ses relevés intacts. C'est le parti déjà pris pour `livraison` et pour
/// `envois` : `VERSION` ne bouge pas, le champ est facultatif.
#[test]
fn un_livrable_d_avant_s_ouvre_jamais_genere() {
    let avant = r#"
[ozalid]
version = 5
[livre]
titre = "Candide"
auteur = "Voltaire"
genre = "roman"
[livraison]
courant = "bod-135x215-broche-creme-90"
[[livraison.livrables]]
pod = "bod"
format = "135x215"
reliure = "broche"
papier = "creme-90"
dos_mm = 18.4
"#;
    let m: Metadonnees = toml::from_str(avant).expect("TOML illisible");
    let l = &m.livraison.livrables[0];
    assert_eq!(l.generation, Generation::Jamais);
    assert_eq!(l.dos_mm, Some(18.4), "le relevé doit traverser intact");
}

/// L'aller-retour dans le fichier : ce qu'une génération a laissé se relit tel quel.
/// Le test porte sur TOML et non sur JSON parce que c'est TOML que le `.ozalid` écrit —
/// et parce que TOML exige que les valeurs précèdent les tables, ce qui décide de la
/// place du champ dans la structure.
#[test]
fn l_etat_de_generation_traverse_le_fichier() {
    // `Metadonnees` ne dérive pas `Default` ; `livre()` est le helper que `mod tests`
    // porte déjà, vers la ligne 1681.
    let mut m = Projet::nouveau(livre(), String::new()).meta;
    m.livraison.livrables[0].generation = Generation::Fait {
        interieur: "aaaa".into(),
        couverture: "bbbb".into(),
    };
    let ecrit = toml::to_string(&m).expect("TOML inécrivable");
    let relu: Metadonnees = toml::from_str(&ecrit).expect("TOML illisible");
    assert_eq!(
        relu.livraison.livrables[0].generation,
        Generation::Fait { interieur: "aaaa".into(), couverture: "bbbb".into() }
    );
}

/// Un échec se retient aussi : la ligne doit pouvoir dire pourquoi elle est rouge après
/// une réouverture, sans quoi il faudrait recomposer pour l'apprendre.
#[test]
fn un_echec_de_generation_traverse_le_fichier() {
    let mut m = Projet::nouveau(livre(), String::new()).meta;
    m.livraison.livrables[0].generation = Generation::Echec {
        message: "typst absent".into(),
    };
    let relu: Metadonnees =
        toml::from_str(&toml::to_string(&m).expect("TOML inécrivable")).expect("TOML illisible");
    assert!(matches!(
        &relu.livraison.livrables[0].generation,
        Generation::Echec { message } if message == "typst absent"
    ));
}

/// Un livrable jamais généré n'écrit rien dans le fichier : le `.ozalid` d'un projet
/// neuf ne doit pas grossir d'une table vide par livrable.
#[test]
fn jamais_genere_ne_s_ecrit_pas() {
    let m = Projet::nouveau(livre(), String::new()).meta;
    let ecrit = toml::to_string(&m).expect("TOML inécrivable");
    assert!(!ecrit.contains("generation"), "{ecrit}");
}
```

- [ ] **Étape 2 : voir les tests échouer**

Run : `cd src-tauri && cargo test generation`
Attendu : ÉCHEC à la compilation, `cannot find type 'Generation'`.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `projet.rs`, au-dessus de `Livrable` :

```rust
/// Ce qu'une génération a laissé sur un livrable.
///
/// Les deux empreintes sont celles d'`empreinte::interieur` et d'`empreinte::couverture`
/// **au moment où les fichiers ont été écrits**. Les comparer à celles de l'état courant
/// dit ce qui a bougé depuis — c'est tout le mécanisme, et il n'en faut pas d'autre.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "etat", rename_all = "lowercase")]
pub enum Generation {
    /// Aucun fichier écrit : un livrable qu'on vient de déclarer, ou le livrable d'un
    /// `.ozalid` d'avant ce lot.
    #[default]
    Jamais,
    Fait {
        interieur: String,
        couverture: String,
    },
    Echec {
        message: String,
    },
}

impl Generation {
    /// Rien à écrire dans le fichier : `serde` s'en sert pour taire le cas courant.
    pub fn est_jamais(&self) -> bool {
        matches!(self, Generation::Jamais)
    }
}
```

Et, dans `Livrable`, **en dernière position** :

```rust
    /// Ce que la dernière génération a laissé. **Déclaré en dernier**, et ce n'est pas un
    /// hasard : TOML exige que les valeurs d'une table précèdent ses sous-tables, et cet
    /// état-ci s'écrit en table. Le remonter avant `dos_mm` ferait échouer l'écriture du
    /// `.ozalid` — sur un projet réel, pas dans les tests d'un type isolé.
    #[serde(default, skip_serializing_if = "Generation::est_jamais")]
    pub generation: Generation,
```

Compléter `Livrable::pour` (`projet.rs:318`) avec `generation: Generation::Jamais`. C'est le
seul constructeur littéral de `Livrable` hors tests ; `cargo test` signalera les autres, tous
dans des `mod tests`.

- [ ] **Étape 4 : voir les tests passer**

Run : `cd src-tauri && cargo test`
Attendu : tous passés, y compris les quatre neufs.

Si `l_etat_de_generation_traverse_le_fichier` échoue sur « values must be emitted before
tables », c'est que le champ n'est pas en dernier : c'est exactement ce que le commentaire
ci-dessus annonce.

- [ ] **Étape 5 : commit**

```bash
cd src-tauri && cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test ; cd ..
git add src-tauri/src/projet.rs
git commit -m "Un livrable retient ce que sa génération a laissé, ou pourquoi elle a échoué"
```

---

### Tâche 4 : La question que l'écran posera

**Fichiers :**
- Modifier : `src-tauri/src/empreinte.rs`
- Test : même fichier

**Interfaces :**
- Consomme : `interieur`, `couverture` (tâche 2) ; `Generation` (tâche 3).
- Produit : `pub enum Etat { Jamais, Echec, AJour, Perime { interieur: bool, couverture: bool } }`
  et `pub fn etat(projet: &Projet, l: &Livrable) -> Etat`.

- [ ] **Étape 1 : écrire les tests qui échouent**

```rust
/// Les quatre réponses, sur le même projet. C'est cette fonction que l'écran du lot 3
/// interrogera pour marquer une ligne, et le message qu'il affichera dépend de *laquelle*
/// des deux moitiés a bougé — d'où le couple de booléens plutôt qu'un simple « périmé ».
#[test]
fn l_etat_dit_laquelle_des_deux_moities_a_bouge() {
    let mut p = projet_d_essai();
    let l = p.meta.livraison.livrables[0].clone();

    assert_eq!(etat(&p, &l), Etat::Jamais, "rien n'a été généré");

    let mut a_jour = l.clone();
    a_jour.generation = crate::projet::Generation::Fait {
        interieur: interieur(&p, &l),
        couverture: couverture(&p, &l),
    };
    assert_eq!(etat(&p, &a_jour), Etat::AJour);

    // La couverture seule bouge : l'intérieur écrit reste bon, et le lot 2 s'en servira
    // pour ne pas recomposer 258 pages afin de changer une image.
    let mut q = p.clone();
    q.images.insert("premiere.jpg".into(), vec![1, 2, 3]);
    assert_eq!(
        etat(&q, &a_jour),
        Etat::Perime { interieur: false, couverture: true }
    );

    // Le manuscrit bouge : l'intérieur est faux, et la couverture aussi dès que la
    // pagination aura été reprise — mais tant qu'elle ne l'a pas été, la mesure retenue
    // n'a pas changé, et seule la moitié intérieure est en cause.
    p.texte.push_str("\n\nUn paragraphe de plus.");
    assert_eq!(
        etat(&p, &a_jour),
        Etat::Perime { interieur: true, couverture: false }
    );
}

/// Un échec retenu ne se compare pas : il n'y a pas d'empreinte à confronter, et la ligne
/// doit dire « ça n'a pas marché », pas « c'est périmé ».
#[test]
fn un_echec_retenu_ne_se_compare_pas() {
    let p = projet_d_essai();
    let mut l = p.meta.livraison.livrables[0].clone();
    l.generation = crate::projet::Generation::Echec { message: "typst absent".into() };
    assert_eq!(etat(&p, &l), Etat::Echec);
}
```

- [ ] **Étape 2 : voir les tests échouer**

Run : `cd src-tauri && cargo test empreinte::tests::l_etat`
Attendu : ÉCHEC à la compilation, `cannot find type 'Etat'`.

- [ ] **Étape 3 : écrire l'implémentation**

```rust
/// Où en est un livrable, comparé à l'état courant du projet.
///
/// `Serialize` parce que le lot 2 le fera descendre dans la vue que le front consomme ;
/// il n'a aucun autre usage côté Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "etat", rename_all = "lowercase")]
pub enum Etat {
    /// Jamais généré : rien à regarder, rien à refaire tant qu'on ne l'a pas demandé.
    Jamais,
    /// La dernière génération a échoué ; son message est dans `Generation::Echec`.
    Echec,
    AJour,
    Perime { interieur: bool, couverture: bool },
}

/// Où en est ce livrable.
///
/// Deux empreintes recalculées à chaque appel, sans cache. Hacher le manuscrit du témoin
/// coûte quelques dixièmes de milliseconde là où composer coûte des secondes ; un cache
/// achèterait ce dixième-là au prix d'une invalidation à tenir juste — le même arbitrage
/// que `commands::envoi_vignettes` a déjà tranché dans ce sens.
pub fn etat(projet: &Projet, l: &Livrable) -> Etat {
    let (i, c) = match &l.generation {
        crate::projet::Generation::Jamais => return Etat::Jamais,
        crate::projet::Generation::Echec { .. } => return Etat::Echec,
        crate::projet::Generation::Fait {
            interieur: i,
            couverture: c,
        } => (i, c),
    };
    let (di, dc) = (*i != interieur(projet, l), *c != couverture(projet, l));
    if di || dc {
        Etat::Perime {
            interieur: di,
            couverture: dc,
        }
    } else {
        Etat::AJour
    }
}
```

- [ ] **Étape 4 : voir les tests passer**

Run : `cd src-tauri && cargo test`
Attendu : tous passés.

- [ ] **Étape 5 : le témoin, puis commit**

```bash
cd src-tauri && cargo run --example temoin
```
Attendu : **le même compte de pages qu'avant le lot** — 98 sur le premier gabarit BoD du
témoin. Un écart est une régression ; devant un écart, relancer d'abord après
`touch pods maquettes src/lib.rs` (le piège des ressources embarquées, `CLAUDE.md`).

```bash
cd src-tauri && cargo fmt && cargo clippy --all-targets -- -D warnings ; cd ..
git add src-tauri/src/empreinte.rs
git commit -m "Un livrable sait dire laquelle de ses deux moitiés a bougé"
```

---

## À l'œil, avant de clore le lot

Rien. C'est le propre de ce lot : aucun pixel ne change, et tout ce qu'il pose se prouve au
compilateur et aux tests. La première chose à regarder viendra au lot 3.

## Ce que ce lot ne fait pas

- Il ne fait rien descendre vers le front : `Etat` est calculable, personne ne l'appelle
  encore. C'est le lot 2 qui le fera entrer dans la vue.
- Il ne touche à aucune commande, à aucun bouton, à aucun fichier de `src/`.
- Il ne réutilise aucun intérieur : la mutualisation sur disque est le cœur du lot 2, et
  elle s'appuiera sur `Etat::Perime { interieur: false, .. }` posé ici.
- Il n'efface rien sur le disque.
