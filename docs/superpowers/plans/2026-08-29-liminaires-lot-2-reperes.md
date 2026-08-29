# Liminaires lot 2 — Les repères, sans la table

> **Pour les agents :** SOUS-SKILL REQUIS — `superpowers:subagent-driven-development`
> (recommandé) ou `superpowers:executing-plans` pour exécuter ce plan tâche par tâche. Les
> étapes sont cochables (`- [ ]`).

**But :** poser à l'ouverture de chaque pièce du livre un repère `#metadata` étiqueté, que
la table des matières du lot 3 interrogera — **sans rien afficher et sans déplacer une seule
page**. Le lot ne se voit nulle part : sa preuve est le témoin inchangé.

**Architecture :** une fonction `repere(&Piece)` rend un `#metadata((rang, numero, titre))`
étiqueté `<ozalid-tdm>`, posé aux **quatre** ouvertures de pièce que l'intérieur compose —
la pièce liminaire dans `liminaires()`, la page de partie, le chapitre et l'annexe dans
`assemble()`. Chaque repère est écrit **après** le saut de page qui ouvre la pièce, et pour
la page de partie **à l'intérieur** de son `#page(footer: none)[…]` : c'est ce qui fait que
`.location().page()` rendra le folio de l'ouverture, et non celui de la page d'avant.

**Pile :** Rust (Tauri 2, serde), sidecar Typst 0.15.1, tests `cargo test` (dont
`-- --ignored` pour ceux qui composent pour de vrai).

**Spec :** `docs/superpowers/specs/2026-08-28-liminaires-et-mentions-design.md`, section 2
(§ 2.3 la mécanique, § 2.4 la neutralité, § 4 le risque central).

## Contraintes globales

- **Le témoin ne bouge pas : 98 pages, dos 7,21 mm**, et 118 pages pour la seconde
  fabrication. C'est **le livrable du lot**, pas une observation d'accompagnement — spec
  § 2.4. Un écart d'une page invalide le procédé.
- **Typst est épinglé en 0.15.1.** Ce lot ne la touche pas.
- **`VERSION` ne bouge pas** (`src-tauri/src/projet.rs:58`, vaut 5). Aucun champ de projet
  n'est ajouté par ce lot : le réglage `Table` est du lot 3.
- **Rien ne change côté front, ni dans le README.** Il n'y a rien à montrer : c'est le sens
  de « les repères, sans la table » (spec § 6, lot 2). Ne pas anticiper le lot 3.
- **Français** dans les commentaires et les commits ; messages de commit en phrase
  descriptive, sans préfixe conventionnel — le dépôt écrit « Le pavé compose la ligne
  blanche… », jamais « feat: … ».
- **Tout test neuf doit avoir été vu échouer.** TDD quand le test précède le code ; mutation
  ciblée et **lancée** quand le code existe déjà. Les mutations exactes sont écrites dans les
  tâches — les poser, voir le rouge, les retirer. Un test jamais rouge ne protège rien.
- **`cargo clippy --all-targets -- -D warnings` échoue déjà sur la baseline**, sur
  `src-tauri/src/police.rs:123`, depuis rustc 1.98. **Ce n'est pas la faute de ce lot** : ne
  pas le corriger ici, ne pas s'en accuser. Vérifier seulement qu'aucun avertissement **neuf**
  n'apparaît.

## Décisions arbitrées — ne pas les rouvrir

1. **Les repères sont posés toujours**, table allumée ou éteinte (spec § 2.4). Ne les poser
   que sous réglage aurait rendu la preuve impossible : allumer la table changerait alors
   deux choses à la fois, et un écart de pagination n'aurait plus de coupable identifiable.
2. **`repere()` vit dans `interieur.rs`**, à côté de `ouverture_piece()`, et non dans un
   module neuf. C'est une fonction de dix lignes qui sert le même fichier ; un module `tdm.rs`
   se décidera au lot 3 si la composition de la table le mérite.
3. **Trois champs, pas un libellé prémâché** : `rang`, `numero`, `titre`. La table du lot 3
   compose ses deux rangs et ses points de conduite à partir de là. Fabriquer la ligne dès
   ici enfermerait la mise en forme dans le Rust, alors qu'elle appartient à Typst.
4. **Le titre voyage brut, échappé en chaîne** (`echappe_chaine`), pas en markup : une valeur
   de `metadata` est une chaîne, pas du contenu composé.

## Invariants sur lesquels ce plan s'appuie

- **Le dépôt sait déjà qu'un `metadata` étiqueté ne coûte rien** : `src-tauri/src/typst.rs:18`
  pose `MARQUEUR = "#context [#metadata(counter(page).final().first()) <pages>]"` en fin de
  chaque source, et le témoin est stable à 98 pages avec lui. C'est un précédent, **pas une
  preuve** pour un repère posé au milieu du flux : la preuve est la tâche 3.
- **`Typst::pages`** (`typst.rs:57`) et **`Typst::mesures`** (`typst.rs:81`) lancent
  `typst eval query(<…>)` sans produire de PDF. Les tests du lot s'en servent tels quels :
  **aucune API neuve n'est à ajouter à `Typst`**. En production, Rust ne lit jamais les
  repères — c'est Typst qui les interrogera, au lot 3, par `context query(<ozalid-tdm>)`.
- **Le premier chapitre s'ouvre en page 5**, ou en page 7 quand le livre porte une dédicace
  (`interieur.rs:341-343`, commentaire du corps). Ce fait sert d'ancre au test de folio.
- **`echappe_chaine` est déjà importé** dans `interieur.rs:15`. Rien à ajouter aux `use`.
- **Les quatre ouvertures** et rien d'autre : `interieur.rs:620` (pièce liminaire),
  `interieur.rs:372` (page de partie), `interieur.rs:387` (chapitre), `interieur.rs:411`
  (annexe). Les numéros de ligne sont ceux de `3f3394e` et bougeront d'une tâche à l'autre :
  se repérer sur le code cité, pas sur le numéro.

---

### Tâche 1 : Le repère se fabrique

**Fichiers :**
- Modifier : `src-tauri/src/interieur.rs` — une constante et une fonction, juste après
  `ouverture_piece()` (≈ ligne 665)
- Test : `src-tauri/src/interieur.rs`, module `mod tests` du même fichier

**Interfaces :**
- Consomme : `Piece`, `Sorte`, `echappe_chaine` — déjà importés (`interieur.rs:15`)
- Produit : `pub const TDM: &str = "ozalid-tdm";` et `fn repere(p: &Piece) -> String`, une
  ligne Typst terminée par `\n`. Le lot 3 lira `TDM` pour écrire sa requête ; la tâche 2 de
  ce lot appellera `repere` aux quatre ouvertures.

- [ ] **Étape 1 : écrire les tests, qui ne compilent pas encore**

À ajouter dans `mod tests`, sous une section neuve placée avant
`/* ---------- le témoin de l'invariant, composé pour de vrai ---------- */` :

```rust
    /* ---------- les repères de table ---------- */

    /// Ce qu'un repère porte, sorte par sorte. Les quatre lignes de ce test sont les
    /// quatre cas que `Sorte` admet : la table du lot 3 n'aura rien d'autre à composer.
    ///
    /// Le rang n'est pas décoratif — c'est lui qui indente. Une `Partie` rendue au
    /// second rang mettrait la partie au niveau de ses propres chapitres, et la table
    /// mentirait sur la structure du livre.
    #[test]
    fn chaque_sorte_porte_son_rang_son_numero_et_son_titre() {
        let cas = [
            (
                Sorte::Partie("II".into()),
                "Seconde",
                r#"#metadata((rang: 1, numero: "II", titre: "Seconde"))<ozalid-tdm>"#,
            ),
            (
                Sorte::Chapitre(7),
                "Le vent",
                r#"#metadata((rang: 2, numero: "7", titre: "Le vent"))<ozalid-tdm>"#,
            ),
            (
                Sorte::Liminaire,
                "Préface",
                r#"#metadata((rang: 2, numero: "", titre: "Préface"))<ozalid-tdm>"#,
            ),
            (
                Sorte::Annexe,
                "Postface",
                r#"#metadata((rang: 2, numero: "", titre: "Postface"))<ozalid-tdm>"#,
            ),
        ];
        for (sorte, titre, attendu) in cas {
            let p = Piece {
                sorte: sorte.clone(),
                titre: titre.into(),
                blocs: vec![],
            };
            assert_eq!(
                repere(&p).trim_end(),
                attendu,
                "le repère de {sorte:?} ne dit pas ce que la table lira"
            );
        }
    }

    /// Un chapitre sans titre est un cas admis du format (`## 7`). La table ne fabrique
    /// aucun libellé que le livre n'imprime pas : elle n'aura que le numéro à composer,
    /// et le titre vide est ce qui le lui dit.
    #[test]
    fn une_piece_sans_titre_laisse_le_titre_vide() {
        let p = Piece {
            sorte: Sorte::Chapitre(7),
            titre: String::new(),
            blocs: vec![],
        };
        assert_eq!(
            repere(&p).trim_end(),
            r#"#metadata((rang: 2, numero: "7", titre: ""))<ozalid-tdm>"#
        );
    }

    /// Un guillemet dans un titre refermerait la chaîne du dictionnaire, et la source
    /// ne composerait plus — le même piège que la page de titre, déjà tenu par
    /// `echappe_chaine`. Ici la faute serait pire : elle casserait la composition d'un
    /// livre dont le seul tort est d'avoir un titre à guillemets.
    #[test]
    fn un_titre_a_guillemets_ne_referme_pas_le_dictionnaire_du_repere() {
        let p = Piece {
            sorte: Sorte::Liminaire,
            titre: "L'« ouverture » dite\nen deux temps".into(),
            blocs: vec![],
        };
        let s = repere(&p);
        assert!(
            s.contains(r#"titre: "L'« ouverture » dite\nen deux temps""#),
            "titre mal cité : {s}"
        );
        assert_eq!(s.lines().count(), 1, "le repère tient sur une ligne : {s}");
    }
```

- [ ] **Étape 2 : lancer, vérifier le rouge**

```bash
cd src-tauri && cargo test --lib interieur::tests::chaque_sorte 2>&1 | tail -20
```

Attendu : **échec de compilation**, `cannot find function 'repere' in this scope`. C'est le
rouge de la TDD ; ne pas passer à l'étape suivante sans l'avoir vu.

- [ ] **Étape 3 : écrire la constante et la fonction**

Dans `src-tauri/src/interieur.rs`, immédiatement après `fn ouverture_piece(…)` et avant
`fn titre_sous_numero(…)` :

```rust
/// L'étiquette que porte chaque repère de table, telle qu'une requête Typst la nomme.
///
/// Publique parce que la table la lira — `context query(<ozalid-tdm>)` — et qu'un nom
/// recopié à deux endroits est un nom qui divergera.
pub const TDM: &str = "ozalid-tdm";

/// Le repère qu'une pièce laisse à l'ouverture de sa page, pour la table des matières.
///
/// **Il ne s'affiche pas et n'occupe aucune place** : un `metadata` n'est pas mis en
/// page, il est seulement situé. C'est ce qui permet de le poser dans tous les livres,
/// table allumée ou non, et de prouver par le témoin qu'il ne coûte rien — une preuve
/// impossible si la pose dépendait du réglage, puisque l'allumer changerait alors deux
/// choses à la fois.
///
/// Trois champs, et non un libellé prémâché : le rang indente, le numéro et le titre
/// sont ce que la page d'ouverture imprime. Composer la ligne ici enfermerait la mise
/// en forme dans le Rust, alors qu'elle appartient à la table.
///
/// Les valeurs sont **citées, non composées** : `echappe_chaine`, jamais `echappe`.
fn repere(p: &Piece) -> String {
    let (rang, numero) = match &p.sorte {
        // Une partie tient le premier rang ; tout le reste est indenté sous elle.
        Sorte::Partie(romain) => (1, romain.as_str()),
        Sorte::Chapitre(_) => (2, ""),
        Sorte::Liminaire | Sorte::Annexe => (2, ""),
    };
    let numero = match &p.sorte {
        Sorte::Chapitre(n) => n.to_string(),
        _ => numero.to_string(),
    };
    format!(
        "#metadata((rang: {rang}, numero: \"{}\", titre: \"{}\"))<{TDM}>\n",
        echappe_chaine(&numero),
        echappe_chaine(&p.titre)
    )
}
```

- [ ] **Étape 4 : simplifier le double `match`**

Le code de l'étape 3 lit la sorte deux fois — clippy ne s'en plaindra pas, un relecteur si.
Le remplacer par un seul `match` :

```rust
fn repere(p: &Piece) -> String {
    let (rang, numero) = match &p.sorte {
        // Une partie tient le premier rang ; tout le reste est indenté sous elle.
        Sorte::Partie(romain) => (1, romain.clone()),
        Sorte::Chapitre(n) => (2, n.to_string()),
        Sorte::Liminaire | Sorte::Annexe => (2, String::new()),
    };
    format!(
        "#metadata((rang: {rang}, numero: \"{}\", titre: \"{}\"))<{TDM}>\n",
        echappe_chaine(&numero),
        echappe_chaine(&p.titre)
    )
}
```

- [ ] **Étape 5 : lancer, vérifier le vert**

```bash
cd src-tauri && cargo test --lib interieur::tests 2>&1 | tail -5
```

Attendu : les trois tests neufs passent, aucun autre ne tombe.

- [ ] **Étape 6 : format, clippy, commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

`clippy` échouera sur `src-tauri/src/police.rs:123` — **baseline connue, pas ce lot**.
Vérifier qu'aucun avertissement ne cite `interieur.rs`.

```bash
git add src-tauri/src/interieur.rs
git commit -m "Chaque pièce sait dire son rang, son numéro et son titre à la table"
```

---

### Tâche 2 : Les repères se posent aux quatre ouvertures

**Fichiers :**
- Modifier : `src-tauri/src/interieur.rs` — `liminaires()` (la boucle finale sur les pièces),
  et dans `assemble()` les bras `Sorte::Partie`, `Sorte::Chapitre` et la boucle des annexes
- Test : `src-tauri/src/interieur.rs`, module `mod tests`

**Interfaces :**
- Consomme : `repere(&Piece) -> String` et `TDM`, de la tâche 1
- Produit : une source d'intérieur qui porte un repère par pièce, dans l'ordre du manuscrit.
  Rien d'autre ne change de signature — `assemble` garde ses sept arguments.

- [ ] **Étape 1 : écrire les tests, et les voir rouges**

Ajouter à la section `/* ---------- les repères de table ---------- */` créée en tâche 1 :

```rust
    /// Un manuscrit qui exerce les quatre ouvertures que l'intérieur compose. L'ordre
    /// est celui que `decoupe` impose : liminaires, corps, annexes.
    fn pieces_des_quatre_sortes() -> Vec<Piece> {
        vec![
            Piece {
                sorte: Sorte::Liminaire,
                titre: "Préface".into(),
                blocs: vec![Bloc::Paragraphe("Avant.".into())],
            },
            Piece {
                sorte: Sorte::Partie("I".into()),
                titre: "Première".into(),
                blocs: vec![],
            },
            Piece {
                sorte: Sorte::Chapitre(1),
                titre: "Un".into(),
                blocs: vec![Bloc::Paragraphe("Texte.".into())],
            },
            Piece {
                sorte: Sorte::Chapitre(2),
                titre: "Deux".into(),
                blocs: vec![Bloc::Paragraphe("Encore.".into())],
            },
            Piece {
                sorte: Sorte::Annexe,
                titre: "Postface".into(),
                blocs: vec![Bloc::Paragraphe("Après.".into())],
            },
        ]
    }

    fn source_des_quatre_sortes() -> String {
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        source(
            &livre(),
            &Interieur::default(),
            provider("bod").unwrap(),
            &r,
            &pieces_des_quatre_sortes(),
            None,
        )
    }

    /// Chaque pièce laisse son repère, et une seule fois. Une pièce oubliée — la
    /// postface, la page de partie — manquerait dans la table sans que rien ne le dise,
    /// et c'est exactement le défaut que la spec refuse : « une préface qui a sa page
    /// d'ouverture et n'apparaît pas dans la table serait un défaut ».
    #[test]
    fn les_quatre_sortes_laissent_chacune_leur_repere_dans_l_ordre() {
        let s = source_des_quatre_sortes();
        let reperes: Vec<&str> = s.lines().filter(|l| l.contains(TDM)).collect();
        assert_eq!(
            reperes,
            vec![
                r#"#metadata((rang: 2, numero: "", titre: "Préface"))<ozalid-tdm>"#,
                r#"#metadata((rang: 1, numero: "I", titre: "Première"))<ozalid-tdm>"#,
                r#"#metadata((rang: 2, numero: "1", titre: "Un"))<ozalid-tdm>"#,
                r#"#metadata((rang: 2, numero: "2", titre: "Deux"))<ozalid-tdm>"#,
                r#"#metadata((rang: 2, numero: "", titre: "Postface"))<ozalid-tdm>"#,
            ],
            "les repères de la source ne suivent pas le manuscrit"
        );
    }

    /// **Le repère se pose après le saut de page, jamais avant.** Écrit avant, il serait
    /// situé sur la dernière page de la pièce précédente : la table afficherait un folio
    /// d'une page trop tôt, et le lecteur ouvrirait à la fin du chapitre d'avant. Rien
    /// ne le signalerait — ni le compte de pages, ni le rendu, seulement un livre faux.
    #[test]
    fn le_repere_d_un_chapitre_suit_le_saut_de_page_qui_l_ouvre() {
        let s = source_des_quatre_sortes();
        assert!(
            s.contains(
                "#pagebreak()\n#metadata((rang: 2, numero: \"2\", titre: \"Deux\"))<ozalid-tdm>"
            ),
            "le repère du chapitre 2 n'est pas collé derrière son saut de page :\n{s}"
        );
        assert!(
            s.contains("#pagebreak()\n#metadata((rang: 2, numero: \"\", titre: \"Postface\"))"),
            "le repère de l'annexe n'est pas collé derrière son saut de page :\n{s}"
        );
    }

    /// La page de partie est composée par `#page(footer: none)[…]`, qui rompt le flux de
    /// lui-même. Le repère doit vivre **dedans** : posé avant, il serait situé sur la
    /// page précédente ; posé après, sur la blanche du verso. Dans les deux cas la table
    /// enverrait le lecteur à côté de la page de partie.
    #[test]
    fn le_repere_d_une_partie_vit_dans_sa_page() {
        let s = source_des_quatre_sortes();
        assert!(
            s.contains(
                "#page(footer: none)[\n#metadata((rang: 1, numero: \"I\", titre: \"Première\"))<ozalid-tdm>\n#v(22mm)"
            ),
            "le repère de la partie n'ouvre pas sa page :\n{s}"
        );
    }
```

```bash
cd src-tauri && cargo test --lib interieur::tests::les_quatre_sortes interieur::tests::le_repere 2>&1 | tail -30
```

Attendu : **trois échecs**, `reperes` vide pour le premier, `assert!` faux pour les deux
autres. Aucun repère n'est encore posé.

- [ ] **Étape 2 : poser le repère de la pièce liminaire**

Dans `fn liminaires(…)`, la boucle finale — ajouter la première ligne :

```rust
    for p in pieces {
        s.push_str(&repere(p));
        s.push_str(&ouverture_piece(&p.titre, int.ouverture_piece));
```

- [ ] **Étape 3 : poser le repère de la page de partie, dans sa page**

Dans `fn assemble(…)`, bras `Sorte::Partie(r)` — le `format!` devient :

```rust
                s.push_str(&format!(
                    "#page(footer: none)[\n{}#v(22mm)\n\
                     #align(center, text(size: {}pt)[{r}])\n",
                    repere(p),
                    int.numero
                ));
```

- [ ] **Étape 4 : poser le repère du chapitre, après le saut de page**

Dans le bras `Sorte::Chapitre(numero)`, entre le `if i > 0 && !apres_page { … }` et le
`format!` du numéro :

```rust
                if i > 0 && !apres_page {
                    s.push_str("#pagebreak()\n");
                }
                s.push_str(&repere(p));
                s.push_str(&format!(
                    "#v(22mm)\n#align(center, text(size: {}pt)[{numero}])\n",
                    int.numero
                ));
```

- [ ] **Étape 5 : poser le repère de l'annexe**

Dans la boucle des annexes de `assemble(…)` :

```rust
        for (i, p) in annexes.iter().enumerate() {
            if i > 0 {
                s.push_str("#pagebreak()\n");
            }
            s.push_str(&repere(p));
            s.push_str(&ouverture_piece(&p.titre, int.ouverture_piece));
```

- [ ] **Étape 6 : lancer, vérifier le vert — et que rien d'autre n'a bougé**

```bash
cd src-tauri && cargo test 2>&1 | tail -15
```

Attendu : tout au vert. Les tests de source existants (`les_defauts_reproduisent…`,
`chaque_role_typographique_prend_sa_taille`, `une_page_de_partie_prend_une_belle_page…`)
cherchent des sous-chaînes ; si l'un d'eux tombe, **c'est une régression de pose**, pas un
test à relâcher — lire ce qu'il attendait avant de toucher quoi que ce soit.

- [ ] **Étape 7 : le témoin, seul livrable du lot**

```bash
cd src-tauri && cargo run --example temoin 2>&1 | tail -20
```

Attendu : **98 pages / dos 7,21 mm**, puis **118 pages**. En cas d'écart de valeur du type
`left: 18.75, right: 18.8` — signature d'un catalogue périmé, pas d'une pagination —,
relancer après :

```bash
cd src-tauri && touch pods maquettes src/lib.rs && cargo run --example temoin 2>&1 | tail -20
```

Si le témoin bouge vraiment d'une page, **ne pas continuer** : le `metadata` n'est pas neutre
dans le flux. Parade nommée par la spec § 4, dans l'ordre : (a) accoler le repère à ce qui le
suit sur la même ligne — `repere()` rend sa chaîne sans `\n` final, et les tests d'ancrage de
l'étape 1 cherchent alors `"#pagebreak()\n#metadata(…"` sans le saut, `"…<ozalid-tdm>#v(22mm)"`
après ; (b) si le témoin bouge encore, le procédé est faux : arrêter, et rouvrir la spec avant
d'écrire la moindre ligne de table.

- [ ] **Étape 8 : format, clippy, commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings 2>&1 | grep -c interieur.rs
```

Attendu : `0` — l'échec sur `police.rs:123` reste la baseline connue.

```bash
git add src-tauri/src/interieur.rs
git commit -m "Chaque ouverture de pièce laisse son repère, sans rien montrer"
```

---

### Tâche 3 : La neutralité et l'ancrage, prouvés en composant

**Fichiers :**
- Test : `src-tauri/src/interieur.rs`, section
  `/* ---------- le témoin de l'invariant, composé pour de vrai ---------- */`, à la suite de
  `un_envoi_ne_cree_aucune_page_ou_qu_il_se_pose`

**Interfaces :**
- Consomme : `typst_de_test()`, `pages_de()`, `page_rendue()`, `manuscrit_long()`,
  `pieces_des_quatre_sortes()` (tâche 2), `TDM` (tâche 1) — toutes déjà en place
- Produit : rien pour la production. Deux tests `#[ignore]`, qui sont **la preuve du lot**.

Les deux tests composent pour de vrai : ils sont marqués `#[ignore]` comme leur voisin et se
lancent par `cargo test -- --ignored`.

- [ ] **Étape 1 : écrire le test de neutralité**

```rust
    /// **La preuve du lot, et son seul livrable.** Les repères ne déplacent aucune page
    /// et ne se voient sur aucune.
    ///
    /// Compter les `#metadata` dans la source ne prouverait rien : c'est Typst qui décide
    /// de la mise en page, et un élément « invisible » qui ouvrirait un paragraphe
    /// ajouterait un espacement — donc, sur un livre entier, des pages. La pagination
    /// change alors le dos, donc la planche, et les exemplaires partent avec une
    /// couverture fausse sans que rien ne le signale.
    ///
    /// La référence est la **même source privée de ses repères**, ligne à ligne : la
    /// seule différence entre les deux documents est ce que ce lot ajoute. Comparer
    /// chaque page rendue, et pas seulement le compte, ferme la porte au cas où deux
    /// écarts se compenseraient.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn les_reperes_n_occupent_aucune_place_et_ne_se_voient_nulle_part() {
        let typst = typst_de_test();
        let dossier = tempfile::tempdir().expect("répertoire de travail");
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let r = Reglage {
            gouttiere: pr.gouttieres[0].2,
            blanche: false,
        };
        let avec = source(
            &livre(),
            &Interieur::default(),
            pr,
            &r,
            &manuscrit_long(),
            None,
        );
        let sans: String = avec
            .lines()
            .filter(|l| !l.contains(TDM))
            .collect::<Vec<_>>()
            .join("\n");
        assert_ne!(avec, sans, "la source ne porte aucun repère : rien n'est prouvé");

        let n_avec = pages_de(&typst, dossier.path(), "avec", &avec);
        let n_sans = pages_de(&typst, dossier.path(), "sans", &sans);
        assert!(n_sans > 30, "manuscrit trop court pour prouver : {n_sans}");
        assert_eq!(
            n_avec, n_sans,
            "les repères ont déplacé la pagination : {n_avec} au lieu de {n_sans}"
        );
        for page in 1..=n_sans {
            assert_eq!(
                page_rendue(&typst, dossier.path(), "avec", page),
                page_rendue(&typst, dossier.path(), "sans", page),
                "un repère se voit sur la page {page}"
            );
        }
    }
```

- [ ] **Étape 2 : voir ce test rouge par mutation**

Le code existe déjà : le rouge s'obtient en rendant le repère visible. Dans `repere()`,
remplacer `#metadata((…))` par un texte composé — mutation temporaire :

```rust
    format!(
        "#text(size: 1pt)[.]#metadata((rang: {rang}, numero: \"{}\", titre: \"{}\"))<{TDM}>\n",
        echappe_chaine(&numero),
        echappe_chaine(&p.titre)
    )
```

```bash
cd src-tauri && cargo test --lib -- --ignored les_reperes_n_occupent 2>&1 | tail -20
```

Attendu : **échec** sur « un repère se voit sur la page … ». **Retirer la mutation**, relancer,
vérifier le vert.

- [ ] **Étape 3 : écrire le test d'ancrage des folios**

```rust
    /// **Le repère est situé sur la page qu'il ouvre**, et c'est tout ce qui fera la
    /// justesse des folios de la table au lot 3. Posé un cran trop tôt — avant le saut
    /// de page —, il enverrait le lecteur à la fin de la pièce précédente ; rien dans le
    /// compte de pages ni dans le rendu ne le dirait.
    ///
    /// Le manuscrit est fait de chapitres d'une page : les folios attendus sont donc
    /// **consécutifs**, à partir de la page 5 — celle où le corps s'ouvre quand le livre
    /// n'a pas de dédicace (`assemble`, commentaire du corps). Un repère mal ancré rend
    /// deux fois le même folio, et la suite cesse d'être consécutive.
    ///
    /// Les folios sont relevés par `Typst::mesures`, qui lit `<mesures>` sans composer de
    /// PDF : la source de test publie ce que la table interrogera, sans qu'aucune API
    /// neuve n'entre dans le code de production — la table, elle, lira ses repères depuis
    /// Typst même.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn chaque_repere_est_situe_sur_la_page_qu_il_ouvre() {
        let typst = typst_de_test();
        let dossier = tempfile::tempdir().expect("répertoire de travail");
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let r = Reglage {
            gouttiere: pr.gouttieres[0].2,
            blanche: false,
        };
        let pieces: Vec<Piece> = manuscrit_long().into_iter().take(6).collect();
        let mut s = source(&livre(), &Interieur::default(), pr, &r, &pieces, None);
        // Le folio de chaque repère, indexé par son rang d'apparition : `mesures` rend
        // un dictionnaire de nombres, c'est exactement ce qu'il faut.
        s.push_str(
            "\n#context [#metadata(query(<ozalid-tdm>).enumerate().fold((:), (d, it) => \
             d + ((str(it.at(0))): it.at(1).location().page())))<mesures>]\n",
        );
        let chemin = dossier.path().join("ancrage.typ");
        std::fs::write(&chemin, &s).expect("source non écrite");
        let folios = typst.mesures(&chemin).expect("mesures refusées");

        let releves: Vec<f64> = (0..pieces.len())
            .map(|i| {
                *folios
                    .get(&i.to_string())
                    .unwrap_or_else(|| panic!("aucun repère au rang {i} : {folios:?}"))
            })
            .collect();
        assert_eq!(
            releves,
            vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            "les repères ne suivent pas l'ouverture des chapitres"
        );
    }
```

- [ ] **Étape 4 : lancer le test d'ancrage**

```bash
cd src-tauri && cargo test --lib -- --ignored chaque_repere_est_situe 2>&1 | tail -25
```

Attendu : vert. **S'il est rouge sur les valeurs** — par exemple `[5, 7, 9, …]` —, les
chapitres de `manuscrit_long()` ne tiennent pas sur une page dans ce gabarit : remplacer le
vecteur attendu par la suite consécutive réellement observée **seulement si elle est
consécutive** (`n, n+1, n+2, …`). Deux folios identiques, eux, sont le défaut que le test
cherche : ne rien ajuster, corriger la pose.

- [ ] **Étape 5 : voir le test d'ancrage rouge par mutation**

Dans `assemble()`, bras `Sorte::Chapitre`, déplacer le repère **avant** le saut de page —
la faute exacte que ce test existe pour attraper :

```rust
                s.push_str(&repere(p));
                if i > 0 && !apres_page {
                    s.push_str("#pagebreak()\n");
                }
```

```bash
cd src-tauri && cargo test --lib -- --ignored chaque_repere_est_situe 2>&1 | tail -20
```

Attendu : **échec**, avec des folios qui se répètent (`[5.0, 5.0, 6.0, …]`). **Rétablir la
pose**, relancer, vérifier le vert.

- [ ] **Étape 6 : la suite complète, ignorés compris, et le témoin**

```bash
cd src-tauri && cargo fmt --check && cargo test 2>&1 | tail -5
cd src-tauri && cargo test -- --ignored 2>&1 | tail -8
cd /Users/jean-paulgavini/Documents/Dev/OzalidStudio && node --test tests/*.test.js 2>&1 | tail -5
cd src-tauri && cargo run --example temoin 2>&1 | tail -20
```

Attendu : tout au vert, témoin à **98 pages / dos 7,21 mm** puis **118 pages**.

- [ ] **Étape 7 : commit**

```bash
git add src-tauri/src/interieur.rs
git commit -m "Les repères ne coûtent rien et pointent juste, composition à l'appui"
```

---

## Ce que ce lot ne fait pas

Rappel, pour qu'aucune tâche ne déborde (spec § 6) : **le réglage `Table` à trois états, la
douzième taille, la composition de la table, la belle page et le PDF ebook qui suit sont le
lot 3.** Ce lot ne touche ni `Interieur`, ni le front, ni le README, ni `epub.rs`. Si une
tâche semble en avoir besoin, c'est qu'elle a débordé.

## Vérification de fin de lot

- `cargo fmt --check` propre
- `cargo clippy --all-targets -- -D warnings` : aucun avertissement citant `interieur.rs`
  (l'échec sur `police.rs:123` est la baseline, voir contraintes globales)
- `cargo test` au vert, et `cargo test -- --ignored` au vert
- `node --test tests/*.test.js` au vert — inchangé par ce lot, mais un rouge dirait qu'on a
  débordé sur le front
- `cargo run --example temoin` : **98 pages / dos 7,21 mm**, puis **118 pages**
- Les trois mutations des tâches 1 à 3 ont été **posées, lancées, vues rouges, retirées**

## Ce qu'aucun test ne verra, et qui se regarde

Rien à l'œil pour ce lot : il n'affiche rien. C'est précisément pourquoi sa vérification est
entièrement mécanique — et pourquoi le témoin en est le seul livrable.
