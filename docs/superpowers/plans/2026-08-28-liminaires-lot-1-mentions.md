# Liminaires lot 1 — Les mentions se citent

> **Pour les agents :** SOUS-SKILL REQUIS — `superpowers:subagent-driven-development`
> (recommandé) ou `superpowers:executing-plans` pour exécuter ce plan tâche par tâche. Les
> étapes sont cochables (`- [ ]`).

**But :** faire que l'ISBN, le dépôt légal et le nom de l'imprimeur s'écrivent **une seule
fois** et paraissent d'eux-mêmes dans le pavé de copyright de la page 4, au lieu d'y être
retapés à côté du code-barres qui les compose déjà.

**Architecture :** trois jetons rejoignent les six de `gabarit.rs`, et la substitution passe
d'un `&Livre` à un `Contexte` — le livre plus l'imprimeur, quand la composition en vise un.
`assemble` reçoit ce contexte **à la place** du livre, argument pour argument : il en a déjà
sept, et clippy refuse le huitième. Là où rien n'est imprimé — l'ebook, la couverture —
l'imprimeur vaut `None` et `%IMPRIMEUR%` rend la chaîne vide, jamais le jeton littéral.

**Pile :** Rust (Tauri 2, serde, toml), front vanilla sans bundler, tests `cargo test` et
`node --test`.

**Spec :** `docs/superpowers/specs/2026-08-28-liminaires-et-mentions-design.md`, section 1.

## Contraintes globales

- **Typst est épinglé en 0.15.1.** Ce lot ne compose rien de neuf ; il ne doit pas la
  toucher.
- **Le témoin ne bouge pas : 98 pages, dos 7,21 mm**, et 118 pages pour la seconde
  fabrication (`bod-120x190-broche-photo-brillant-130`, gouttière 18,0 mm). Aucune page
  n'est ajoutée par ce lot — c'est sa preuve, et un écart signifie que quelque chose a
  bougé qui ne devait pas.
- **`VERSION` ne bouge pas** (`projet.rs:58`, vaut 5). Le champ neuf naît avec son
  `#[serde(default)]`, comme `titre_page` avant lui.
- **Français** dans l'interface, les commentaires et les commits.
- **Tout test neuf doit avoir été vu échouer** — TDD, ou mutation ciblée quand le rouge ne
  s'obtient pas autrement. Un test jamais rouge ne protège rien.

## Décisions arbitrées (brainstorming du 28/08) — ne pas les rouvrir

1. **Les mentions passent par des jetons, imprimeur compris.** Le pavé de copyright reste un
   champ **libre** : la page 4 n'a pas la même forme d'un éditeur à l'autre, et une page
   typée aurait remplacé cette liberté au lieu de l'étendre.
2. **Une seule porte de substitution.** Deux fonctions — l'une avec l'imprimeur, l'autre
   sans — auraient tôt ou tard laissé un champ libre passer par la mauvaise.
3. **`%IMPRIMEUR%` rend la chaîne vide** partout où rien n'est imprimé. Un `%IMPRIMEUR%`
   resté en toutes lettres dans le pavé serait une faute visible du lecteur.
4. **`depot_legal` est saisi, jamais dérivé de l'année courante** : c'est la date d'un acte
   administratif, pas celle de la compilation. Un défaut calculé se lirait comme une mesure —
   le même argument que celui qui interdit de préremplir un fond perdu relevé.
5. **Aucune page n'est ajoutée.** Le pavé de copyright est déjà calé au bas du verso de la
   page de titre, qui est la place française des mentions légales.

## Invariants sur lesquels ce plan s'appuie

- **`Provider::pod_nom` existe déjà** (`catalogue.rs:735`), et pour cette raison-là : il a
  été mis sur le `Provider` pour que la fiche de téléversement n'ait rien à retraduire. Le
  même argument vaut ici. Ne pas le retrouver par `catalogue::resout`.
- **`cle_gabarit()` porte déjà le POD** (`catalogue.rs:915`, `pod-format-reliure`). Le
  partage d'intérieur ne joue qu'entre papiers et finitions d'un même imprimeur, où le nom
  est constant : **rien du partage ne bouge dans ce lot**.
- **`assemble` a exactement sept arguments** (`interieur.rs:260`). Un huitième déclencherait
  `clippy::too_many_arguments` sous `-D warnings`, et le dépôt vient tout juste de retirer
  son dernier `#[expect]` de cette famille : ne pas en remettre un. C'est pourquoi le
  `Contexte` **remplace** `livre` au lieu de s'y ajouter.
- **`substituer` fait une seule passe** (`gabarit.rs:47`), de gauche à droite, et ne relit
  jamais ce qu'un jeton a produit. Ce n'est pas une garde contre les cycles — il ne peut pas
  y en avoir —, c'est une garde contre la relecture de la sortie. Ne pas la remplacer par une
  boucle de `replace`.
- **Aucun jeton n'est préfixe d'un autre** aujourd'hui, et `starts_with` gagne sur le premier
  trouvé. Si un jeton futur en préfixait un autre, l'ordre de la table déciderait — les trois
  ajoutés ici ne sont dans ce cas ni entre eux ni avec les six existants.

## Avant chaque commit

```
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd .. && node --test tests/*.test.js
```

Et, dès qu'un fichier de `src-tauri/` a changé :

```
cd src-tauri && cargo run --example temoin
```

## Pièges transverses

- **Le front est embarqué dans le binaire à la compilation.** Après un changement de `src/`
  seul, `touch src-tauri/src/lib.rs` avant `cargo build`, sinon le binaire garde l'ancien
  front. Vaut pour les tâches 4 et 5.
- **`package.rs` ne bouge pas dans ce lot.** Si l'exécution l'y amène malgré tout, lancer
  aussi `cargo test -- --ignored` (5 tests, sidecar Typst) : `package` ne doit pas interroger
  le catalogue, et `cargo test` seul ne le voit pas.
- **Ne pas toucher `couverture.rs:1383-1385`** (la lecture de l'ISBN pour le code-barres) :
  ce lot ajoute une **lecture de plus** du même champ, il ne déplace pas celle qui existe.

## Structure des fichiers

| Fichier | Ce qu'il gagne |
|---|---|
| `src-tauri/src/projet.rs` | le champ `depot_legal`, et l'argument `imprimeur` sur les cinq méthodes de champs libres |
| `src-tauri/src/gabarit.rs` | la struct `Contexte`, les trois jetons, la signature de `substituer` |
| `src-tauri/src/interieur.rs` | `assemble` et `liminaires` prennent le contexte ; `source` le remplit, `source_ebook` non |
| `src-tauri/src/ebook.rs` | `libres` passe `None` |
| `src-tauri/src/couverture.rs` | `corps_quatre` passe `None` |
| `src/index.html` | le champ de saisie du dépôt légal |
| `src/app.js` | lecture, écriture et câblage du champ |
| `tests/contrats.test.js` | le contrat de départ du champ |
| `README.md` | la liste des jetons et la mention du dépôt légal |

---

### Tâche 1 : Le livre porte son dépôt légal

**Fichiers :**
- Modifier : `src-tauri/src/projet.rs:66-105` (la struct `Livre`) et `src-tauri/src/projet.rs:180-196` (`vide()`)
- Test : `src-tauri/src/projet.rs`, dans `mod tests`, à la suite de
  `un_projet_sans_titre_de_page_recoit_le_jeton` (`projet.rs:1119`)

**Interfaces :**
- Produit : `Livre::depot_legal: String` — vide par défaut, lu et écrit tel quel.
- Consommé par : la tâche 2 (`%DEPOT_LEGAL%`) et la tâche 4 (la saisie).

- [ ] **Étape 1 : écrire le test qui échoue**

Dans `mod tests` de `src/projet.rs` :

```rust
    /// Le dépôt légal est saisi, et un `.ozalid` d'avant ce lot n'en porte pas. Il doit
    /// s'ouvrir avec un champ vide plutôt que d'être refusé : `VERSION` ne bouge pas, donc
    /// rien ne migre, donc c'est `serde` qui doit tenir l'absence.
    #[test]
    fn un_projet_sans_depot_legal_s_ouvre_avec_un_champ_vide() {
        let toml = r#"
[ozalid]
version = 5

[livre]
titre = "Les Heures creuses"
auteur = "Ivan Pjig"
"#;
        let m: Metadonnees = toml::from_str(toml).expect("projet sans dépôt légal refusé");
        assert_eq!(m.livre.depot_legal, "");
    }
```

- [ ] **Étape 2 : le lancer et le voir échouer**

```
cd src-tauri && cargo test un_projet_sans_depot_legal -- --nocapture
```

Attendu : **échec de compilation**, `no field 'depot_legal' on type 'Livre'`.

- [ ] **Étape 3 : ajouter le champ**

Dans la struct `Livre`, à la suite de `isbn` :

```rust
    /// Le dépôt légal, tel qu'il paraît sur la page de mentions : « septembre 2026 ».
    ///
    /// **Saisi, jamais dérivé de l'année courante.** C'est la date d'un acte
    /// administratif, pas celle de la compilation, et un défaut calculé se lirait comme
    /// une mesure — le même argument qui interdit de préremplir un fond perdu relevé.
    ///
    /// Vide est le cas courant : un tirage privé ne dépose rien, et le pavé de copyright
    /// saute la ligne devenue vide.
    #[serde(default)]
    pub depot_legal: String,
```

Et dans `vide()`, à la suite de `isbn: String::new(),` :

```rust
            depot_legal: String::new(),
```

- [ ] **Étape 4 : le test passe**

```
cd src-tauri && cargo test un_projet_sans_depot_legal
```

Attendu : PASS.

- [ ] **Étape 5 : poser la mutation, la voir rougir, la retirer**

Retirer le `#[serde(default)]` du champ, relancer `cargo test un_projet_sans_depot_legal` :
le test doit **échouer** sur `missing field 'depot_legal'`. Remettre l'attribut, relancer,
PASS. C'est ce qui prouve que ce test protège la relecture d'un `.ozalid` ancien, et non la
seule existence du champ.

- [ ] **Étape 6 : vérifier et commiter**

```
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd .. && node --test tests/*.test.js
cd src-tauri && cargo run --example temoin
```

Le témoin doit annoncer 98 pages / dos 7,21 mm et 118 pages sur la seconde fabrication.

```bash
git add src-tauri/src/projet.rs
git commit -m "Le livre porte la date de son dépôt légal, saisie et jamais calculée"
```

---

### Tâche 2 : La substitution reçoit un contexte, et trois jetons de plus

**Fichiers :**
- Modifier : `src-tauri/src/gabarit.rs:9-47` (le type `Jeton`, la table `JETONS`, `substituer`)
- Modifier : `src-tauri/src/projet.rs:198-226` (les cinq méthodes de champs libres)
- Modifier les appelants : `src-tauri/src/interieur.rs:567,587,594`,
  `src-tauri/src/ebook.rs:236`, `src-tauri/src/couverture.rs:1401,1434`
- Modifier les tests appelants : `src-tauri/src/projet.rs:1131,1141,1152,1157,1172,1173,1596,1614,1615,1703`, `src-tauri/src/import.rs:582`
- Test : `src-tauri/src/gabarit.rs`, dans `mod tests`

**Interfaces :**
- Produit :
  ```rust
  pub struct Contexte<'a> { pub livre: &'a Livre, pub imprimeur: Option<&'a str> }
  pub fn substituer(texte: &str, ctx: &Contexte) -> String
  ```
  et sur `Livre`, cinq signatures qui gagnent le même argument :
  ```rust
  pub fn titre_page(&self, imprimeur: Option<&str>) -> String
  pub fn copyright(&self, imprimeur: Option<&str>) -> String
  pub fn prix(&self, imprimeur: Option<&str>) -> String
  pub fn mention(&self, imprimeur: Option<&str>) -> String
  pub fn dedicace(&self, imprimeur: Option<&str>) -> Option<String>
  ```
- **À cette tâche, tous les appelants passent `None`.** L'intérieur ne connaît son imprimeur
  qu'à la tâche 3 : ici, le comportement observable ne change pas, et c'est voulu — un lot
  qui changerait la plomberie *et* le rendu ne dirait pas lequel des deux a bougé.

- [ ] **Étape 1 : écrire les tests qui échouent**

Dans `mod tests` de `src/gabarit.rs`, un helper et quatre tests :

```rust
    fn ctx<'a>(l: &'a Livre, imprimeur: Option<&'a str>) -> Contexte<'a> {
        Contexte { livre: l, imprimeur }
    }

    #[test]
    fn l_isbn_et_le_depot_legal_se_citent() {
        let l = Livre {
            isbn: "978-2-07-041311-9".into(),
            depot_legal: "septembre 2026".into(),
            ..Livre::vide()
        };
        assert_eq!(
            substituer("ISBN %ISBN% — dépôt légal : %DEPOT_LEGAL%", &ctx(&l, None)),
            "ISBN 978-2-07-041311-9 — dépôt légal : septembre 2026"
        );
    }

    /// L'imprimeur ne vient pas du livre : il vient de ce qu'on fabrique. Le même livre
    /// composé chez deux imprimeurs porte deux mentions.
    #[test]
    fn l_imprimeur_vient_du_contexte_pas_du_livre() {
        let l = livre();
        assert_eq!(
            substituer("Imprimé par %IMPRIMEUR%", &ctx(&l, Some("BoD"))),
            "Imprimé par BoD"
        );
    }

    /// Sans imprimeur — l'ebook, la couverture —, le jeton rend la **chaîne vide**, jamais
    /// lui-même. Un `%IMPRIMEUR%` en toutes lettres dans le pavé de copyright serait une
    /// faute que le lecteur verrait, sur un fichier que plus personne ne relit.
    #[test]
    fn sans_imprimeur_le_jeton_s_efface_au_lieu_de_rester() {
        let l = livre();
        assert_eq!(substituer("Imprimé par %IMPRIMEUR%.", &ctx(&l, None)), "Imprimé par .");
    }

    /// Un ISBN vide n'écrit rien, pas le jeton : c'est le cas d'un tirage privé, et il est
    /// légitime.
    #[test]
    fn un_isbn_vide_n_ecrit_rien() {
        let l = Livre { isbn: String::new(), ..Livre::vide() };
        assert_eq!(substituer("%ISBN%", &ctx(&l, None)), "");
    }
```

- [ ] **Étape 2 : les lancer et les voir échouer**

```
cd src-tauri && cargo test --lib gabarit::
```

Attendu : **échec de compilation** — `cannot find type 'Contexte'`.

- [ ] **Étape 3 : la porte unique**

Dans `src/gabarit.rs`, remplacer le type `Jeton` et la table, et poser la struct :

```rust
/// Ce contre quoi un champ libre se résout : le livre, et l'imprimeur quand la
/// composition en vise un.
///
/// **Une seule porte.** Garder `substituer(&str, &Livre)` et ajouter une seconde fonction
/// pour l'imprimeur aurait ouvert deux chemins de substitution, dont le second serait tôt
/// ou tard oublié sur un champ libre — et l'oubli ne se verrait qu'imprimé.
pub struct Contexte<'a> {
    pub livre: &'a Livre,
    /// `None` quand rien n'est imprimé : l'ebook, la couverture. `%IMPRIMEUR%` rend alors
    /// la chaîne vide, jamais le jeton littéral.
    pub imprimeur: Option<&'a str>,
}

/// Un jeton et ce qu'il désigne dans le contexte.
type Jeton = (&'static str, fn(&Contexte) -> &str);

/// Les jetons reconnus, et ce que chacun désigne.
///
/// Les six premiers sont des clés du livre, littérales par définition : aucune n'est
/// elle-même substituée, et c'est ce qui rend toute référence cyclique impossible. Les
/// trois derniers ne changent rien à cette propriété — l'ISBN et le dépôt légal sont des
/// clés comme les autres, et l'imprimeur ne vient pas du livre du tout.
const JETONS: [Jeton; 9] = [
    ("%TITRE%", |c| &c.livre.titre),
    ("%AUTEUR%", |c| &c.livre.auteur),
    ("%GENRE%", |c| &c.livre.genre),
    ("%EDITEUR%", |c| &c.livre.editeur),
    ("%COLLECTION%", |c| &c.livre.collection),
    ("%MONOGRAMME%", |c| &c.livre.monogramme),
    ("%ISBN%", |c| &c.livre.isbn),
    ("%DEPOT_LEGAL%", |c| &c.livre.depot_legal),
    ("%IMPRIMEUR%", |c| c.imprimeur.unwrap_or("")),
];
```

Puis la signature, sans toucher au corps de la boucle sinon l'appel de la valeur :

```rust
pub fn substituer(texte: &str, ctx: &Contexte) -> String {
```

et, dans le bras `Some((jeton, valeur))` :

```rust
                sortie.push_str(valeur(ctx));
```

- [ ] **Étape 4 : les cinq méthodes du livre**

Dans `src/projet.rs`, chacune gagne l'argument et construit le contexte :

```rust
    /// Titre tel qu'il doit paraître sur la page de titre, jetons résolus.
    pub fn titre_page(&self, imprimeur: Option<&str>) -> String {
        crate::gabarit::substituer(
            &self.titre_page,
            &crate::gabarit::Contexte { livre: self, imprimeur },
        )
    }
```

Faire de même pour `copyright`, `prix`, `mention` et `dedicace`, en conservant le corps de
chacune pour le reste — `dedicace` garde son rognage et son `Option`.

- [ ] **Étape 5 : les appelants passent `None`**

`interieur.rs:567` → `livre.titre_page(None)` ; `interieur.rs:587` → `livre.copyright(None)` ;
`interieur.rs:594` → `livre.dedicace(None)` ; `ebook.rs:236` → les trois avec `None` ;
`couverture.rs:1434` → `livre.mention(None)` et `livre.prix(None)` ; `couverture.rs:1401` →

```rust
    let resume = crate::gabarit::substituer(
        &q.texte,
        &crate::gabarit::Contexte { livre, imprimeur: None },
    );
```

**La couverture reste sans imprimeur, et c'est un choix** : `corps_quatre` n'a pas le
`Provider` en portée, et le nom de l'imprimeur n'a rien à faire sur une 4ème. Le jeton s'y
efface, comme dans l'ebook.

Puis les tests appelants listés en tête de tâche : chacun gagne `None`.

- [ ] **Étape 6 : les tests passent**

```
cd src-tauri && cargo test --lib gabarit:: && cargo test
```

Attendu : PASS, et **aucun test existant modifié dans son intention** — seul l'argument
change.

- [ ] **Étape 7 : poser la mutation, la voir rougir, la retirer**

Remplacer `c.imprimeur.unwrap_or("")` par `c.imprimeur.unwrap_or("%IMPRIMEUR%")` : le test
`sans_imprimeur_le_jeton_s_efface_au_lieu_de_rester` doit **échouer**. Remettre, PASS.

- [ ] **Étape 8 : vérifier et commiter**

```
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd .. && node --test tests/*.test.js
cd src-tauri && cargo run --example temoin
```

Témoin inchangé : 98 pages / 7,21 mm, et 118 pages.

```bash
git add src-tauri/src/gabarit.rs src-tauri/src/projet.rs src-tauri/src/interieur.rs \
        src-tauri/src/ebook.rs src-tauri/src/couverture.rs src-tauri/src/import.rs
git commit -m "Un champ libre se résout contre un contexte : le livre, et l'imprimeur s'il y en a un"
```

---

### Tâche 3 : L'intérieur imprimé connaît son imprimeur

**Fichiers :**
- Modifier : `src-tauri/src/interieur.rs:260-268` (`assemble`), `:507-517` (`source`),
  `:527-539` (`source_ebook`), `:547` (`liminaires`)
- Test : `src-tauri/src/interieur.rs`, dans `mod tests`

**Interfaces :**
- Consomme : `gabarit::Contexte` (tâche 2), `Provider::pod_nom` (`catalogue.rs:735`).
- Produit : `assemble(ctx: &Contexte, int, pr, r, pieces, envoi, avant)` — sept arguments,
  le contexte **à la place** du livre. `source` construit
  `Contexte { livre, imprimeur: Some(&pr.pod_nom) }` ; `source_ebook` construit
  `Contexte { livre, imprimeur: None }`.

- [ ] **Étape 1 : écrire les tests qui échouent**

Dans `mod tests` de `src/interieur.rs` :

```rust
    /// Le pavé de copyright de la page 4 cite l'imprimeur, et l'imprimeur vient du
    /// gabarit visé : c'est la même mécanique que le dos, où le chiffre ne passe jamais
    /// par un humain.
    #[test]
    fn le_copyright_de_l_interieur_cite_l_imprimeur_du_gabarit() {
        let mut l = livre();
        l.copyright = "Imprimé par %IMPRIMEUR%".into();
        let pr = provider("bod").unwrap();
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        let s = source(&l, &Interieur::default(), pr, &r, &[], None);
        assert!(s.contains(&pr.pod_nom), "le nom de l'imprimeur manque à la page 4");
        assert!(!s.contains("%IMPRIMEUR%"), "le jeton est resté littéral");
    }

    /// L'ebook n'est pas imprimé : le jeton s'y efface. Le même livre, la même page 4,
    /// et une mention qui n'a pas de sens sur un écran.
    #[test]
    fn l_ebook_n_a_pas_d_imprimeur() {
        let mut l = livre();
        l.copyright = "Imprimé par %IMPRIMEUR%".into();
        let pr = provider("bod").unwrap();
        let s = source_ebook(&l, &Interieur::default(), pr, &[], "");
        assert!(!s.contains(&pr.pod_nom), "l'ebook a nommé un imprimeur");
        assert!(!s.contains("%IMPRIMEUR%"), "le jeton est resté littéral");
    }
```

**Note d'exécution :** `livre()` est le helper déjà présent dans ce `mod tests`, et
`provider("bod")` le helper de test du catalogue (`catalogue.rs:880`, clé plate historique
inscrite en `catalogue.rs:928`). Le `Reglage` se construit sur place et les pièces valent
`&[]` : c'est exactement ce que fait
`la_source_porte_le_gabarit_de_l_imprimeur_et_le_marqueur`, le test voisin.

- [ ] **Étape 2 : les lancer et les voir échouer**

```
cd src-tauri && cargo test --lib interieur::tests::le_copyright_de_l_interieur
cd src-tauri && cargo test --lib interieur::tests::l_ebook_n_a_pas_d_imprimeur
```

Attendu : le premier **échoue** — la page 4 ne contient pas « BoD », le jeton s'y est effacé
(tâche 2, tous les appelants passaient `None`). Le second **passe déjà**, et c'est normal :
il garde l'ebook de régresser quand le premier sera réparé. Le noter dans le commit.

- [ ] **Étape 3 : le contexte remplace le livre dans `assemble`**

```rust
fn assemble(
    ctx: &Contexte,
    int: &Interieur,
    pr: &Provider,
    r: &Reglage,
    pieces: &[Piece],
    envoi: Option<Trace>,
    avant: Option<&str>,
) -> String {
```

Le corps lit `ctx.livre` là où il lisait `livre`, et passe `ctx` à `liminaires`. **Ne pas
ajouter un huitième argument** : `assemble` en a sept, et clippy refuse le suivant.

`liminaires` prend le contexte à son tour :

```rust
fn liminaires(ctx: &Contexte, int: &Interieur, pieces: &[Piece]) -> String {
```

et ses trois lectures deviennent `ctx.livre.titre_page(ctx.imprimeur)`,
`ctx.livre.copyright(ctx.imprimeur)`, `ctx.livre.dedicace(ctx.imprimeur)`.

- [ ] **Étape 4 : les deux portes publiques disent ce qu'elles fabriquent**

```rust
pub fn source(
    livre: &Livre,
    int: &Interieur,
    pr: &Provider,
    r: &Reglage,
    pieces: &[Piece],
    envoi: Option<Trace>,
) -> String {
    // Ce qui part à l'impression connaît son imprimeur : c'est le seul endroit où il
    // entre dans le livre, et il ne vient pas du livre mais du gabarit visé.
    let ctx = Contexte { livre, imprimeur: Some(&pr.pod_nom) };
    assemble(&ctx, int, pr, r, pieces, envoi, None)
}
```

```rust
pub fn source_ebook(
    livre: &Livre,
    int: &Interieur,
    pr: &Provider,
    pieces: &[Piece],
    couverture: &str,
) -> String {
    let r = Reglage { gouttiere: pr.exterieur, blanche: false };
    // Aucun imprimeur : le format vient d'un gabarit, mais rien n'est imprimé.
    let ctx = Contexte { livre, imprimeur: None };
    assemble(&ctx, int, pr, &r, pieces, None, Some(couverture))
}
```

- [ ] **Étape 5 : les tests passent**

```
cd src-tauri && cargo test --lib interieur::
```

Attendu : PASS, les deux.

- [ ] **Étape 6 : poser la mutation, la voir rougir, la retirer**

Dans `source_ebook`, remplacer `imprimeur: None` par `imprimeur: Some(&pr.pod_nom)` : le test
`l_ebook_n_a_pas_d_imprimeur` doit **échouer**. Remettre `None`, PASS. C'est la seule preuve
que ce test protège quelque chose.

- [ ] **Étape 7 : vérifier et commiter**

```
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd .. && node --test tests/*.test.js
cd src-tauri && cargo run --example temoin
```

Témoin inchangé : le jeton résolu ne change aucune longueur de texte composé au point de
déplacer une page — et s'il l'avait fait, c'est ici qu'on le verrait.

```bash
git add src-tauri/src/interieur.rs
git commit -m "La page 4 nomme son imprimeur, que l'ebook continue de taire"
```

---

### Tâche 4 : Le dépôt légal se saisit

**Fichiers :**
- Modifier : `src/index.html:80-82` (après le champ ISBN)
- Modifier : `src/app.js:595` (lecture), `:1044` (écriture), `:1511-1512` (câblage)
- Test : `tests/contrats.test.js`, à la suite du test de l'ISBN (`:498`)

**Interfaces :**
- Consomme : `Livre::depot_legal` (tâche 1).
- Produit : l'élément `inDepotLegal`, rogné à l'envoi comme l'ISBN, jamais `null`.

- [ ] **Étape 1 : écrire le test qui échoue**

Dans `tests/contrats.test.js` :

```js
/**
 * Le dépôt légal part **rogné**, en chaîne, jamais `null`.
 *
 * C'est une date d'acte recopiée telle qu'elle sera imprimée — « septembre 2026 » —, que
 * rien dans l'application ne sait reformater. Un front qui enverrait `null` ferait échouer
 * la désérialisation au premier caractère tapé, comme pour l'ISBN.
 */
test('le dépôt légal part rogné, jamais null', async () => {
  let envoye = null;
  const capte = async (cmd, args) => {
    if (cmd === 'livre_modifier') {
      envoye = args.livre;
      return PROJET;
    }
    return invoke(cmd, args);
  };
  const { els } = await charge({ invoke: capte, open: async () => null });

  els.get('inDepotLegal').value = '';
  await els.get('inDepotLegal').declenche('change');
  assert.strictEqual(envoye.depot_legal, '', 'dépôt légal vide envoyé en null');

  els.get('inDepotLegal').value = '  septembre 2026  ';
  await els.get('inDepotLegal').declenche('change');
  assert.strictEqual(envoye.depot_legal, 'septembre 2026', 'espaces gardés');
});
```

- [ ] **Étape 2 : le lancer et le voir échouer**

```
node --test tests/contrats.test.js
```

Attendu : échec — `inDepotLegal` est introuvable dans le faux DOM.

- [ ] **Étape 3 : le champ dans la fenêtre**

Dans `src/index.html`, juste après le `<label>` de l'ISBN :

```html
      <label><span>Dépôt légal</span>
        <input type="text" id="inDepotLegal"
               placeholder="vide : pas de dépôt légal"></label>
```

- [ ] **Étape 4 : le câblage**

Dans `src/app.js`, après `$('inIsbn').value = p.livre.isbn ?? '';` :

```js
  $('inDepotLegal').value = p.livre.depot_legal ?? '';
```

Après `isbn: $('inIsbn').value.trim(),` :

```js
    // Rogné, jamais reformaté : « septembre 2026 » est ce qui s'imprimera, et rien ici ne
    // sait lire une date. Le champ vide part en chaîne vide, pas en null.
    depot_legal: $('inDepotLegal').value.trim(),
```

Et dans la liste des identifiants câblés sur `change`, à la suite de `'inIsbn'` :
`'inDepotLegal',`.

- [ ] **Étape 5 : le test passe**

```
node --test tests/contrats.test.js
```

Attendu : PASS.

- [ ] **Étape 6 : poser la mutation, la voir rougir, la retirer**

Retirer `'inDepotLegal'` de la liste des identifiants câblés : le test doit **échouer** —
rien n'est envoyé. Le remettre, PASS.

- [ ] **Étape 7 : vérifier et commiter**

```
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd .. && node --test tests/*.test.js
```

```bash
git add src/index.html src/app.js tests/contrats.test.js
git commit -m "La date du dépôt légal se saisit sous l'ISBN, à côté de ce qu'elle accompagne"
```

---

### Tâche 5 : Le README dit les neuf jetons, et le lot se regarde

**Fichiers :**
- Modifier : `README.md` (section « 1 · Livre », et la ligne des jetons de la 4ème)

**Interfaces :** aucune — cette tâche ne produit pas de code.

- [ ] **Étape 1 : le README**

Dans « 1 · Livre », à la liste de l'identité, ajouter le dépôt légal ; et écrire, là où le
README parle des jetons de la 4ème, que le pavé de copyright les reconnaît aussi et que
trois d'entre eux ne viennent pas du livre seul :

> Le pavé de copyright reconnaît les mêmes jetons que la 4ème, plus `%ISBN%`,
> `%DEPOT_LEGAL%` et `%IMPRIMEUR%`. Les deux premiers sont saisis une fois dans l'onglet
> Livre ; le troisième vient du livrable, si bien que le même livre composé chez deux
> imprimeurs porte deux mentions sans qu'on ait rien retapé. Dans l'ebook, qui n'est
> imprimé nulle part, `%IMPRIMEUR%` ne rend rien.

Ne pas recopier la liste des jetons dans le HTML : l'aide de la fenêtre est déjà servie par
`gabarit::jetons()`, et une copie mentirait au prochain jeton ajouté.

- [ ] **Étape 2 : la vérification à l'œil, que rien n'automatise**

```
cd src-tauri && touch src/lib.rs && cargo run --example packager
```

Ouvrir l'intérieur produit et **lire la page 4** : le pavé doit porter l'ISBN et le dépôt
légal saisis, et le nom de l'imprimeur du gabarit visé. Puis, dans la fenêtre
(`cargo tauri dev`), écrire `%IMPRIMEUR%` dans le champ Copyright d'un projet de travail et
vérifier que la mention change quand on change de livrable.

- [ ] **Étape 3 : le témoin, une dernière fois**

```
cd src-tauri && cargo run --example temoin
```

**98 pages, dos 7,21 mm ; 118 pages sur la seconde fabrication.** Un écart ici, après un lot
qui n'ajoute aucune page, se relance d'abord après un `touch` de `src-tauri/pods/`, de
`src-tauri/maquettes/` et de `src-tauri/src/lib.rs` — c'est la signature du piège des
ressources embarquées, pas d'une régression.

- [ ] **Étape 4 : commiter**

```bash
git add README.md
git commit -m "Le README dit les trois jetons neufs et d'où chacun vient"
```

---

## À l'œil, avant de clore le lot

1. La page 4 composée porte l'ISBN, le dépôt légal et l'imprimeur, chacun à sa ligne, sans
   ligne vide résiduelle quand l'un des trois manque.
2. Le PDF de l'ebook ne nomme aucun imprimeur, et ne montre aucun `%IMPRIMEUR%`.
3. La 4ème n'a pas changé : le code-barres est au même endroit, l'ISBN en clair au-dessus.
4. Le champ Dépôt légal se remplit, se vide, et l'entête passe à « modifié » dans les deux
   sens.

## Ce que ce lot ne fait pas

- **La table des matières** — lots 2 et 3 du même chantier.
- **Une page de mentions typée** : le pavé reste un champ libre, décision de cadrage 1.
- **L'ours complet** — adresse de l'imprimeur, numéro d'impression : le pavé les accepte en
  texte, aucune donnée typée ne leur est ajoutée.
- **Toucher `VERSION`** : le champ neuf naît avec son défaut, un `.ozalid` ancien s'ouvre.
