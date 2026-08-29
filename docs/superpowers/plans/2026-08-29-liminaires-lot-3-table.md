# Liminaires lot 3 — La table des matières

> **Pour les agents :** SOUS-SKILL REQUIS — `superpowers:subagent-driven-development`
> (recommandé) ou `superpowers:executing-plans` pour exécuter ce plan tâche par tâche. Les
> étapes sont cochables (`- [ ]`).

**But :** composer la table des matières du livre imprimé, à partir des repères que le lot 2
a posés — un réglage à trois états (absente, en tête, en fin), éteint par défaut, une
douzième taille, la belle page, et le PDF ebook qui suit. Allumer la table ajoute des pages,
donc change le dos : c'est le comportement voulu, et c'est ce que ce lot doit prouver.

**Architecture :** un `enum Table` et deux champs de plus dans `Interieur` — le réglage et
la taille d'une entrée. Une fonction `table_matieres(&Interieur) -> String` rend un bloc
`#context query(<ozalid-tdm>)` qui compose ses lignes à partir des repères ; `liminaires()`
l'insère en tête (après le copyright, avant la préface), `assemble()` en fin (après les
annexes). **Typst résout seul l'auto-référence** : la table occupe des pages, et les folios
qu'elle affiche en tiennent compte, en une seule invocation. Aucun aller-retour côté Rust.

**Pile :** Rust (Tauri 2, serde), sidecar Typst 0.15.1, front vanilla, tests `cargo test`
(dont `-- --ignored` pour ceux qui composent pour de vrai) et `node --test`.

**Spec :** `docs/superpowers/specs/2026-08-28-liminaires-et-mentions-design.md`, section 2
(§ 2.1 le réglage, § 2.2 la ligne, § 2.3 la mécanique, § 2.5 belle page et dos), § 3 pour ce
qui bouge ailleurs, § 4 pour les risques, § 5 pour les mutations exigées.

## Contraintes globales

- **Le témoin par défaut ne bouge pas : 98 pages / dos 7,21 mm, puis 118 pages.** La table
  naît absente ; si l'une de ces deux valeurs bouge, c'est que le lot a débordé sur des
  livres qui n'ont rien demandé. **Une troisième fabrication rejoint le témoin**, table
  allumée, avec sa pagination relevée (tâche 5) — c'est elle, et elle seule, qui a le droit
  d'être neuve.
- **Typst est épinglé en 0.15.1.** Ce lot ne la touche pas.
- **`VERSION` ne bouge pas** (`src-tauri/src/projet.rs`, vaut 5). `Interieur` porte déjà
  `#[serde(default)]` sur la structure entière : un `.ozalid` ancien reçoit `Table::Absente`
  et la taille par défaut, c'est-à-dire exactement le livre qu'il composait. Spec § 3.
- **Français** dans l'interface, les commentaires et les commits ; messages de commit en
  phrase descriptive, sans préfixe conventionnel — le dépôt écrit « Les repères ne coûtent
  rien… », jamais « feat: … ».
- **Tout test neuf doit avoir été vu échouer.** TDD quand le test précède le code ; mutation
  ciblée et **lancée** quand le code existe déjà. Les mutations exactes sont écrites dans les
  tâches — les poser, voir le rouge, les retirer.
- **`cargo clippy --all-targets -- -D warnings` échoue déjà sur la baseline**, sur
  `src-tauri/src/police.rs:123`, depuis rustc 1.98. **Ce n'est pas la faute de ce lot** : ne
  pas le corriger ici, ne pas s'en accuser. Vérifier seulement qu'aucun avertissement **neuf**
  n'apparaît.
- **Le front est embarqué à la compilation** : après un changement de `src/` seul,
  `touch src-tauri/src/lib.rs` avant `cargo build`, sinon le binaire garde l'ancien front.

## Décisions arbitrées — ne pas les rouvrir

Les quatre premières ont été tranchées avec l'utilisateur le 29/08/2026, les autres le sont
par ce plan avec leur raison.

1. **En tête, la table vient avant la préface** — après le copyright et la dédicace, avant
   les pièces liminaires du manuscrit. Le lecteur trouve le plan sans traverser un texte, et
   la table annonce la préface elle-même.
2. **Un seul intitulé, « Table des matières »**, dans les deux positions. Pas de « Sommaire »
   en tête : rien à expliquer dans l'interface, et c'est le mot que tout lecteur reconnaît.
3. **La ligne** : le rang 1 (partie) reprend les capitales de sa page d'ouverture, le rang 2
   reste en casse telle ; le numéro et le titre sont séparés par un tiret cadratin espacé
   (« 1 — Le vent ») ; points de conduite ; folio à droite.
4. **Le témoin gagne une troisième fabrication**, table en tête, plutôt qu'une mesure
   consignée dans un document. Une référence qu'on relance est une garde ; une phrase dans un
   plan n'en est pas une.
5. **Deux noms, parce qu'un seul ne pouvait pas servir aux deux.** La spec § 2.1 appelle
   `table` le réglage **et** la douzième taille : deux champs d'une même struct ne peuvent
   pas porter le même nom. Le réglage garde `table` — c'est lui qu'on lit partout —, la
   taille devient **`entree_table`**, « entrée » étant le terme éditorial pour une ligne de
   table. Interface : « Entrée de table des matières ».
6. **L'indentation du rang 2 est conditionnelle** : elle ne paraît que si le livre porte au
   moins une partie. Un roman sans parties — le cas courant, celui du témoin — verrait sinon
   toutes ses lignes décalées de 5 mm sous un rang 1 qui n'existe pas. Relevé à la
   composition le 29/08, pas déduit.
7. **La blanche qui suit la table appartient à l'appelant**, pas à `table_matieres()`. En
   tête c'est le saut de parité qui ouvre la pièce suivante — le dispositif déjà en place
   après chaque pièce liminaire ; en fin c'est la blanche de parité du livre, que `converge`
   pose. Mettre un `pagebreak` de sortie dans la fonction ajouterait une page en fin de
   volume, que rien n'occuperait.
8. **Le message de refus de pagination ne change pas** (`package.rs`, `verifie_pagination`).
   Le risque de la spec § 4 — une table longue qui pousse le livre hors des bornes — est
   couvert par le message existant : « *bod-135x215-broche-creme-90 : 902 pages, hors des 24
   à 900 que BoD accepte en broche.* » Il dit le compte et la borne, il ne prétend pas
   connaître la cause ; lui faire deviner « c'est votre table » supposerait qu'il sache ce
   que le livre pesait sans elle, ce qu'il n'a pas.

## Invariants sur lesquels ce plan s'appuie

**Tous relevés par composition le 29/08/2026** avec le sidecar `typst 0.15.1` du dépôt et les
polices de `src-tauri/fonts` — pas déduits.

- **L'auto-référence converge en une invocation.** Sur un manuscrit de 60 chapitres, la table
  en tête occupe deux pages et les folios sortent consécutifs à partir de 7 : la table s'est
  comptée elle-même. C'est la mécanique de la spec § 2.3, vérifiée avant d'être planifiée.
- **`.location().page()` rend le folio imprimé**, parce que `counter(page)` court depuis le
  faux-titre sans jamais être remis à zéro — seul son affichage est coupé (`foreground`,
  commentaire de `interieur.rs`). **Ce lot n'introduit aucune remise à zéro ni liminaire en
  romain.** S'il en introduisait une, l'équivalence tomberait et la table devrait passer par
  `counter(page).at(loc)`.
- **Les pièces hors folio gardent un folio de table** : une préface en page 7 est annoncée
  « 7 » alors que sa page ne porte aucun numéro imprimé. C'est l'usage — les liminaires sont
  comptés sans être foliotés —, et c'est ce que le lot 2 a rendu possible.
- **Le repère ne distingue pas une pièce liminaire d'une annexe** : les deux donnent
  `rang: 2, numero: ""`. Suffisant pour la table à deux rangs de la spec ; la préface paraît
  donc au même rang qu'un chapitre. Ne pas y toucher : un quatrième champ serait un autre
  lot.
- **`chaque_repere_est_situe_sur_la_page_qu_il_ouvre` ne casse pas.** Le lot 2 le prévoyait
  parce qu'il supposait une table allumée dans les tests ; elle naît éteinte et ce test
  compose avec `Interieur::default()`, donc ses folios `[5, 7, 9, 10, 11]` restent vrais.
  **S'il rougit, c'est une régression, pas un ajustement à faire** — la mémoire du lot 2 sur
  ce point est à corriger, pas à suivre.
- **`les_reperes_n_occupent_aucune_place_et_ne_se_voient_nulle_part` reste vert** pour la même
  raison, et son filtrage par ligne (`!l.contains(TDM)`) continue de fonctionner : ce lot
  n'accole aucun repère à son voisin.
- **Le PDF ebook suivra sans une ligne de code** : `source_ebook` appelle le même `assemble`
  avec le même `&Interieur`. Il n'y a qu'un test à écrire. L'archive EPUB, elle, ne connaît
  pas `Interieur` et garde sa table de navigation native (spec § 3, hors périmètre).
- **Dans l'ebook, la parité est décalée d'une page** : `couverture::page_une` occupe la page
  1, si bien qu'un `pagebreak(to: "odd")` y vise un verso du livre relié. C'est le
  comportement existant depuis les pièces liminaires, pas une conséquence de ce lot : ne pas
  chercher à le corriger ici.
- **Aucune API neuve n'entre dans `Typst`.** `pages`, `mesures` et `apercus` existent et
  suffisent aux tests ; en production, Rust ne lit jamais les repères — c'est Typst qui les
  interroge.

---

### Tâche 1 : Le livre sait s'il porte une table

Le réglage à trois états et la douzième taille, dans `Interieur`. Rien ne compose encore.

**Fichiers :**
- Modifier : `src-tauri/src/interieur.rs` — l'enum après `MAX_PT`, deux champs dans
  `Interieur`, `Default`, `tailles()`
- Test : `src-tauri/src/interieur.rs`, module `mod tests` du même fichier

**Interfaces :**
- Consomme : `serde::{Deserialize, Serialize}` — déjà importés en tête de fichier
- Produit :
  - `pub enum Table { Absente, EnTete, EnFin }`, `Copy`, `Default` sur `Absente`,
    sérialisé en kebab-case (`"absente"`, `"en-tete"`, `"en-fin"`) — le front enverra ces
    chaînes, la tâche 4 s'y branche
  - `Interieur::table: Table` et `Interieur::entree_table: f64` (défaut `9.0`)
  - `Interieur::tailles()` rend désormais **douze** paires

- [ ] **Étape 1 : écrire les tests, qui ne compilent pas encore**

À ajouter dans `mod tests`, à la suite de `une_taille_hors_bornes_est_refusee_et_nommee` :

```rust
    /// La table naît **absente**, et un `.ozalid` écrit avant ce lot la relit absente.
    ///
    /// C'est le même parti que la collection sur le dos : allumée d'office, elle
    /// ajouterait des pages à tous les livres déjà composés, donc changerait leur dos
    /// sans que personne l'ait demandé. `VERSION` n'a pas à bouger pour autant —
    /// `#[serde(default)]` porte sur la structure entière, et un projet ancien reçoit
    /// exactement le livre qu'il composait.
    #[test]
    fn la_table_nait_absente_et_un_projet_ancien_la_relit_absente() {
        assert_eq!(Interieur::default().table, Table::Absente);
        let ancien: Interieur = serde_json::from_str("{}").expect("un projet sans intérieur");
        assert_eq!(ancien.table, Table::Absente, "un .ozalid ancien s'allume tout seul");
        assert_eq!(
            ancien.entree_table,
            Interieur::default().entree_table,
            "la douzième taille manque à un projet ancien"
        );
    }

    /// Les trois états passent la frontière dans la forme que le front envoie.
    ///
    /// Le sélecteur de l'onglet Livre pose `"en-tete"` dans la valeur de son option :
    /// une sérialisation en `"EnTete"` ferait échouer `interieur_modifier` sur un
    /// message de serde, à mi-chemin entre les deux côtés, là où rien ne se lit.
    #[test]
    fn les_trois_etats_de_la_table_se_serialisent_en_kebab() {
        for (etat, attendu) in [
            (Table::Absente, r#""absente""#),
            (Table::EnTete, r#""en-tete""#),
            (Table::EnFin, r#""en-fin""#),
        ] {
            let json = serde_json::to_string(&etat).expect("état sérialisable");
            assert_eq!(json, attendu);
            let relu: Table = serde_json::from_str(&json).expect("état relisible");
            assert_eq!(relu, etat);
        }
    }

    /// La douzième taille est bornée comme les onze autres, et l'erreur la nomme.
    ///
    /// `tailles()` est la seule liste que `verifie()` parcourt : un champ ajouté à la
    /// struct mais oublié dans la liste passerait à 0 pt sans un mot, et Typst
    /// composerait une table invisible dont la pagination donnerait un dos faux.
    #[test]
    fn la_taille_d_entree_de_table_est_bornee_comme_les_autres() {
        let mauvais = Interieur {
            entree_table: 0.0,
            ..Interieur::default()
        };
        let err = mauvais.verifie().unwrap_err();
        assert!(
            err.contains("table des matières"),
            "l'erreur doit nommer le rôle : {err}"
        );
        assert_eq!(Interieur::default().tailles().len(), 12);
    }
```

`serde_json` est déjà une dépendance du crate (`typst.rs` s'en sert) : rien à ajouter au
`Cargo.toml`.

- [ ] **Étape 2 : lancer les tests, les voir refuser de compiler**

```bash
cd src-tauri && cargo test --lib la_table_nait_absente 2>&1 | tail -20
```

Attendu : **échec de compilation**, `cannot find type Table in this scope` et
`no field table on type Interieur`. C'est le rouge de départ.

- [ ] **Étape 3 : écrire l'enum et les deux champs**

Dans `src-tauri/src/interieur.rs`, juste après `pub const MAX_PT: f64 = 48.0;` :

```rust
/// Où la table des matières se compose — ou pas du tout.
///
/// **Absente par défaut**, et pour la raison qui a éteint la collection sur le dos :
/// allumée d'office, elle ajouterait des pages à tous les livres déjà composés, donc
/// changerait leur dos sans que personne l'ait demandé.
///
/// Le réglage vit dans `Interieur` et non dans `Livre` : c'est un choix de composition,
/// qui déplace la pagination comme la police le fait, pas un trait de l'identité du
/// livre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Table {
    #[default]
    Absente,
    EnTete,
    EnFin,
}
```

Dans `pub struct Interieur`, **juste après `pub police: String,`** :

```rust
    /// Où la table des matières se compose. L'allumer ajoute des pages, donc change le
    /// dos : `modifier_interieur` oublie les mesures pour cette raison, comme il le fait
    /// pour la police.
    pub table: Table,
```

et, dans la même struct, **juste après `pub ouverture_piece: f64,`** (les tailles suivent
l'ordre où le livre les rencontre, le folio ferme la marche) :

```rust
    /// Une ligne de la table des matières — celle du titre de la table, lui, est
    /// `ouverture_piece` : la table s'ouvre comme une préface, c'est une pièce du livre.
    pub entree_table: f64,
```

Dans `impl Default for Interieur`, aux mêmes places :

```rust
            table: Table::Absente,
```

```rust
            entree_table: 9.0,
```

Dans `fn tailles`, changer le type de retour en `[(&'static str, f64); 12]` et ajouter,
après la ligne de `ouverture_piece` :

```rust
            ("entrée de table des matières", self.entree_table),
```

Enfin, dans le commentaire de `tailles()`, remplacer « Les onze tailles » par « Les douze
tailles » : il se lit comme un compte, et un compte faux dans un commentaire est une piste
fausse pour le prochain.

- [ ] **Étape 4 : lancer les trois tests**

```bash
cd src-tauri && cargo test --lib la_table_nait_absente les_trois_etats la_taille_d_entree 2>&1 | tail -15
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Attendu : les trois au vert, et la suite entière au vert — aucun test existant ne compare
`Interieur` champ à champ.

- [ ] **Étape 5 : voir le contrôle rouge par mutation**

Retirer la ligne `("entrée de table des matières", self.entree_table),` de `tailles()` — la
faute exacte que ce contrôle existe pour attraper (un champ dans la struct, absent de la
liste que `verifie()` parcourt) :

```bash
cd src-tauri && cargo test --lib la_taille_d_entree 2>&1 | tail -15
```

Attendu : **échec**, sur `l'erreur doit nommer le rôle` puis sur le compte de 12.
**Rétablir la ligne**, relancer, vérifier le vert.

- [ ] **Étape 6 : commit**

```bash
cd src-tauri && cargo fmt && cargo fmt --check
cd /Users/jean-paulgavini/Documents/Dev/OzalidStudio
git add src-tauri/src/interieur.rs
git commit -m "Le livre sait s'il porte une table, et de quelle taille elle se compose"
```

---

### Tâche 2 : La table se compose

Le bloc Typst, et ses deux places dans le volume. Toujours aucune composition réelle : les
tests de cette tâche lisent la source.

**Fichiers :**
- Modifier : `src-tauri/src/interieur.rs` — `TITRE_TABLE` et `table_matieres()` après
  `ouverture_piece()`, l'insertion en tête dans `liminaires()`, l'insertion en fin dans
  `assemble()`
- Test : `src-tauri/src/interieur.rs`, module `mod tests`

**Interfaces :**
- Consomme : `Table`, `Interieur::table`, `Interieur::entree_table` (tâche 1) ;
  `pub const TDM` et `fn ouverture_piece(titre, pt)` (lot 2, déjà là)
- Produit : `fn table_matieres(int: &Interieur) -> String` — le bloc complet, terminé par
  `\n`, **sans** saut de parité de sortie (décision 7). La tâche 3 le composera pour de vrai.

- [ ] **Étape 1 : écrire les tests, qui échouent sur une table qui ne se compose pas**

À ajouter dans `mod tests`, à la fin de la section `/* ---------- les repères de table
---------- */` que le lot 2 a ouverte, sous un intertitre neuf :

```rust
    /* ---------- la table des matières ---------- */

    /// La source des quatre sortes sous un réglage de table donné — `source_des_quatre_sortes`
    /// avec le réglage en plus, mêmes gabarit et gouttière.
    fn source_avec_table(table: Table) -> String {
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        source(
            &livre(),
            &Interieur {
                table,
                ..Interieur::default()
            },
            provider("bod").unwrap(),
            &r,
            &pieces_des_quatre_sortes(),
            None,
        )
    }

    /// **Le livre par défaut ne porte aucune table**, et pas même la requête qui
    /// l'aurait composée. C'est la garde des livres déjà composés : leur dos ne bouge
    /// pas parce que ce lot existe.
    #[test]
    fn une_table_absente_ne_compose_rien() {
        let s = source_avec_table(Table::Absente);
        assert!(
            !s.contains("Table des matières"),
            "un livre sans réglage porte une table\n{s}"
        );
        assert!(
            !s.contains(&format!("query(<{TDM}>)")),
            "un livre sans réglage interroge les repères\n{s}"
        );
    }

    /// En tête, la table vient **après le copyright et avant la préface** : le lecteur
    /// trouve le plan du livre sans traverser un texte, et la table annonce la préface
    /// elle-même. Décision de produit du 29/08 — voir le plan du lot, § décisions.
    #[test]
    fn la_table_en_tete_se_compose_apres_le_copyright_et_avant_la_preface() {
        let s = source_avec_table(Table::EnTete);
        let table = s.find("Table des matières").expect("aucune table composée");
        let copyright = s.find("©").expect("aucun pavé de copyright");
        let preface = s.find("Préface").expect("aucune préface");
        assert!(copyright < table, "la table précède le copyright\n{s}");
        assert!(table < preface, "la table suit la préface\n{s}");
    }

    /// En fin, la table ferme le volume — **après les annexes**, qui font partie du
    /// livre qu'elle indexe.
    #[test]
    fn la_table_en_fin_ferme_le_volume_apres_les_annexes() {
        let s = source_avec_table(Table::EnFin);
        let table = s.find("Table des matières").expect("aucune table composée");
        // L'annexe de `pieces_des_quatre_sortes` s'intitule « Postface » : c'est sa
        // zone qui en fait une annexe, pas son titre.
        let annexe = s.find("Postface").expect("aucune annexe");
        assert!(annexe < table, "la table précède l'annexe\n{s}");
        assert!(
            s[table..].contains(MARQUEUR),
            "la table déborde après le marqueur de fin\n{s}"
        );
    }

    /// **La table ne porte pas l'étiquette des repères**, sous peine de se lister
    /// elle-même — une ligne « Table des matières » dans la table, avec le folio de sa
    /// propre première page.
    ///
    /// Le compte des `#metadata((rang:` est la mesure juste : il vaut le nombre de
    /// pièces, table allumée ou non. Chercher l'absence de l'étiquette ne dirait rien,
    /// puisque la table doit justement l'employer dans sa requête.
    #[test]
    fn la_table_ne_se_liste_pas_elle_meme() {
        let pieces = pieces_des_quatre_sortes();
        for table in [Table::Absente, Table::EnTete, Table::EnFin] {
            let s = source_avec_table(table);
            assert_eq!(
                s.matches("#metadata((rang:").count(),
                pieces.len(),
                "{table:?} : la table s'est ajoutée aux repères\n{s}"
            );
        }
        assert!(
            source_avec_table(Table::EnTete).contains(&format!("query(<{TDM}>)")),
            "la table ne lit pas les repères"
        );
    }

    /// **La table s'ouvre en belle page**, dans les deux positions : une table qui
    /// commence au verso se lit à contre-page, et rien dans le compte de pages ne le
    /// dirait.
    ///
    /// Le saut est un `pagebreak(to: "odd", weak: true)` et non un compte à la main sur
    /// `here().page()` : c'est l'outil que les pièces liminaires emploient déjà, et pour
    /// la même raison — la table est hors folio, donc la page qu'il insère ne porte
    /// aucun numéro.
    #[test]
    fn la_table_s_ouvre_en_belle_page() {
        // Le saut de parité **collé** à l'ouverture de pièce : construite depuis
        // `ouverture_piece`, l'attente ne fige aucun littéral de mise en forme et suit
        // le gabarit de titre si celui-ci bouge.
        let attendu = format!(
            "#pagebreak(to: \"odd\", weak: true)\n{}",
            ouverture_piece(TITRE_TABLE, Interieur::default().ouverture_piece)
        );
        for table in [Table::EnTete, Table::EnFin] {
            let s = source_avec_table(table);
            assert!(
                s.contains(&attendu),
                "{table:?} : la table ne s'ouvre pas en belle page\n{s}"
            );
        }
    }

    /// La taille d'entrée règle les lignes de la table.
    ///
    /// C'est la douzième taille de `tailles()`, et la seule que
    /// `chaque_role_typographique_prend_sa_taille` ne peut pas couvrir — elle ne paraît
    /// dans aucune source tant que le réglage est absent. Le titre de la table, lui,
    /// prend `ouverture_piece`, ce que `la_table_s_ouvre_en_belle_page` vérifie déjà.
    #[test]
    fn la_taille_d_entree_regle_les_lignes_de_la_table() {
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        let s = source(
            &livre(),
            &Interieur {
                table: Table::EnTete,
                entree_table: 22.25,
                ..Interieur::default()
            },
            provider("bod").unwrap(),
            &r,
            &pieces_des_quatre_sortes(),
            None,
        );
        assert!(
            s.contains("set text(size: 22.25pt)"),
            "les lignes de la table ignorent leur taille\n{s}"
        );
    }

    /// **La table affiche le folio du repère qu'elle lit**, sans arithmétique entre les
    /// deux.
    ///
    /// C'est le seul endroit où cette liaison se vérifie. Les tests composés de la tâche
    /// suivante lisent les repères par la même requête que la table, mais **pas ce que
    /// la table imprime** : un `- 2` glissé sur le folio les laisserait tous verts, et
    /// la table renverrait le lecteur deux pages trop tôt sur chaque entrée. Rien dans
    /// le PDF ne le dirait à qui ne compte pas les pages à la main.
    #[test]
    fn la_table_affiche_le_folio_de_chaque_repere() {
        let s = source_avec_table(Table::EnTete);
        assert!(
            s.contains(")#e.location().page()\n"),
            "la table n'affiche pas le folio de son repère, ou le retouche\n{s}"
        );
    }
```

- [ ] **Étape 2 : lancer les tests, les voir rouges**

```bash
cd src-tauri && cargo test --lib table 2>&1 | tail -25
```

Attendu : `une_table_absente_ne_compose_rien` **vert** (rien ne compose encore, c'est
normal) et les **cinq autres rouges** — sur « aucune table composée » pour les trois qui
cherchent le titre, sur « la table ne lit pas les repères », « ignorent leur taille » et
« n'affiche pas le folio » pour les autres. Ce rouge-là est le point de départ : il dit que
la table n'existe pas.

- [ ] **Étape 3 : écrire la table**

Dans `src-tauri/src/interieur.rs`, **juste après `fn ouverture_piece`** et avant la
constante `TDM` :

```rust
/// Le titre que la table porte, dans les deux positions.
///
/// Un seul libellé, et non « Sommaire » en tête : rien à expliquer dans l'interface, et
/// c'est le mot que tout lecteur reconnaît. Décision de produit du 29/08.
const TITRE_TABLE: &str = "Table des matières";
```

puis, **juste après la fonction `repere`** :

```rust
/// La table des matières, composée par Typst depuis les repères que chaque pièce a
/// laissés à l'ouverture de sa page.
///
/// **Typst résout seul l'auto-référence** : la table occupe des pages, et les folios
/// qu'elle affiche en tiennent compte, en une seule invocation. Relevé par composition
/// le 29/08 sur une table de deux pages — les folios sortent consécutifs à partir de la
/// page qui suit la table, pas de celle qui l'aurait suivie sans elle. Les deux voies
/// écartées sont dans la spec § 2.3 : `outline()` natif, que l'intérieur ne peut pas
/// employer faute d'un seul `heading`, et deux passes côté Rust, qui devraient itérer
/// jusqu'au point fixe comme `converge` le fait pour la gouttière.
///
/// **La table ne porte pas l'étiquette `<ozalid-tdm>`** : elle se listerait elle-même.
///
/// Elle s'ouvre en belle page. La blanche qui la suit, quand elle finit sur une impaire,
/// appartient à l'appelant — en tête c'est le saut de parité qui ouvre la pièce
/// suivante, en fin c'est la blanche de parité du livre. Un saut de sortie posé ici
/// ajouterait une page en fin de volume que rien n'occuperait.
///
/// L'indentation du second rang ne paraît que si le livre porte une partie : un roman
/// sans parties verrait sinon toutes ses lignes décalées sous un rang qui n'existe pas.
fn table_matieres(int: &Interieur) -> String {
    let mut s = String::from("#pagebreak(to: \"odd\", weak: true)\n");
    // La table s'ouvre comme une préface : c'est une pièce du livre, et le mot occupe
    // la ligne du numéro.
    s.push_str(&ouverture_piece(TITRE_TABLE, int.ouverture_piece));
    // Le `set par` local défait la justification et l'alinéa du corps : une ligne de
    // table justifiée écarterait ses points de conduite jusqu'à la marge.
    s.push_str(&format!(
        r#"#context {{
  let entrees = query(<{TDM}>)
  let parties = entrees.any(e => e.value.rang == 1)
  set par(justify: false, first-line-indent: 0pt, leading: 0.6em, spacing: 0.6em)
  set text(size: {pt}pt)
  for e in entrees {{
    let v = e.value
    let libelle = if v.numero == "" {{ v.titre }} else if v.titre == "" {{ v.numero }} else {{ v.numero + " — " + v.titre }}
    block(above: if v.rang == 1 {{ 1.2em }} else {{ 0.6em }})[
      #h(if v.rang == 1 or not parties {{ 0mm }} else {{ 5mm }})#if v.rang == 1 {{ upper(libelle) }} else {{ libelle }}#box(width: 1fr, repeat[#h(0.3em).#h(0.3em)])#e.location().page()
    ]
  }}
}}
"#,
        pt = int.entree_table
    ));
    s
}
```

Dans `fn liminaires`, **entre le bloc de la dédicace et la boucle sur les pièces
liminaires** :

```rust
    // La table en tête rejoint la série des liminaires : après le copyright et la
    // dédicace, **avant** la préface. Le lecteur trouve le plan du livre sans traverser
    // un texte, et la table annonce la préface elle-même.
    //
    // `footer: none` court encore ici : la table est hors folio, comme tout ce qui la
    // précède.
    if int.table == Table::EnTete {
        s.push_str(&table_matieres(int));
        // Ce qui suit ouvre en belle page — la préface, ou le corps. Même dispositif
        // qu'après une pièce liminaire, et pour la même raison : la longueur de la table
        // dépend du nombre de pièces, donc d'un manuscrit qu'on retouche.
        s.push_str("#pagebreak(to: \"odd\", weak: true)\n\n");
    }
```

Dans `fn assemble`, **entre le bloc des annexes et la blanche de parité** (`if r.blanche`) :

```rust
    // La table en fin ferme le volume, annexes comprises : c'est la dernière chose du
    // livre. Elle rejoint la zone hors folio que les annexes occupent déjà — et quand il
    // n'y a pas d'annexe, c'est ici que cette zone s'ouvre, dans l'ordre qu'emploie le
    // bloc ci-dessus : le saut de page d'abord, le `set page` ensuite.
    if int.table == Table::EnFin {
        if annexes.is_empty() {
            if !apres_page {
                s.push_str("#pagebreak()\n");
            }
            s.push_str("#set page(footer: none)\n");
        }
        s.push_str(&table_matieres(int));
    }
```

- [ ] **Étape 4 : lancer les tests de la tâche**

```bash
cd src-tauri && cargo test --lib table 2>&1 | tail -25
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Attendu : tout au vert, y compris `aucun_jeton_ne_survit_a_la_source_de_l_interieur` et
`la_source_declare_la_police_du_projet_une_seule_fois`, que la table traverse.

- [ ] **Étape 5 : voir la belle page rouge par mutation**

C'est la mutation que la spec § 5 prescrit — « la table ouverte en page paire ». Dans
`table_matieres`, remplacer le saut d'ouverture par un saut simple :

```rust
    let mut s = String::from("#pagebreak()\n");
```

```bash
cd src-tauri && cargo test --lib la_table_s_ouvre_en_belle_page 2>&1 | tail -15
```

Attendu : **échec** sur « la table ne s'ouvre pas en belle page », pour `EnTete` comme pour
`EnFin`. **Rétablir le saut de parité**, relancer, vérifier le vert.

**Pourquoi ce test lit la source et ne compose pas** : relever la page où la table s'ouvre
demanderait de l'étiqueter, or c'est précisément ce qui lui est interdit — elle se listerait
elle-même. La rendre en PNG pour la reconnaître coûterait deux compositions entières pour
une propriété que le texte porte sans ambiguïté. La composition, elle, prouve ce que le
texte ne peut pas : les folios, en tâche 3.

- [ ] **Étape 6 : voir la table qui se liste elle-même rouge par mutation**

Dans `table_matieres`, coller l'étiquette au titre — la faute que la spec § 2.3 signale :

```rust
    s.push_str(&format!("#metadata((rang: 2, numero: \"\", titre: \"{TITRE_TABLE}\"))<{TDM}>\n"));
    s.push_str(&ouverture_piece(TITRE_TABLE, int.ouverture_piece));
```

```bash
cd src-tauri && cargo test --lib la_table_ne_se_liste_pas 2>&1 | tail -15
```

Attendu : **échec** sur « la table s'est ajoutée aux repères », pour `EnTete` et `EnFin`.
**Retirer la ligne**, relancer, vérifier le vert.

- [ ] **Étape 7 : voir le folio retouché rouge par mutation**

Dans `table_matieres`, glisser une arithmétique entre le repère et ce qui s'imprime — la
faute qu'aucun test composé n'attrapera, puisqu'ils lisent les repères et non la table :

```rust
      ...#box(width: 1fr, repeat[#h(0.3em).#h(0.3em)])#(e.location().page() - 2)
```

```bash
cd src-tauri && cargo test --lib la_table_affiche_le_folio 2>&1 | tail -15
```

Attendu : **échec** sur « la table n'affiche pas le folio de son repère, ou le retouche ».
**Rétablir l'appel nu**, relancer, vérifier le vert.

- [ ] **Étape 8 : mettre à jour le commentaire du test des onze rôles**

`chaque_role_typographique_prend_sa_taille` dit « Onze valeurs distinctes ». La douzième
existe et ce test ne la couvre pas — il ne le peut pas, la table étant absente de sa source.
Ajouter une ligne à son doc-comment, sans toucher au corps :

```rust
    /// La douzième taille — l'entrée de table — ne paraît dans aucune source tant que le
    /// réglage est absent : elle est couverte par
    /// `la_taille_d_entree_regle_les_lignes_et_le_titre_reste_une_ouverture`.
```

- [ ] **Étape 9 : commit**

```bash
cd src-tauri && cargo fmt && cargo fmt --check && cargo clippy --all-targets 2>&1 | grep -c "interieur.rs"
cd /Users/jean-paulgavini/Documents/Dev/OzalidStudio
git add src-tauri/src/interieur.rs
git commit -m "La table se compose depuis les repères, en belle page et sans se lister"
```

Le `grep -c` doit rendre `0` : aucun avertissement neuf sur `interieur.rs`.

---

### Tâche 3 : La table se compte elle-même

La preuve du lot, par composition réelle. Deux tests `#[ignore]`, comme ceux du lot 2.

**Fichiers :**
- Test : `src-tauri/src/interieur.rs`, module `mod tests`, section
  `/* ---------- le témoin de l'invariant, composé pour de vrai ---------- */`

**Interfaces :**
- Consomme : `typst_de_test()`, `pages_de()`, `pieces_des_quatre_sortes()` — tous du lot 2 ;
  `Typst::mesures` (`typst.rs`)
- Produit : rien pour la production. Ces tests sont le livrable.

- [ ] **Étape 1 : écrire les deux tests**

À ajouter **à la fin du module `tests`**, après
`chaque_repere_est_situe_sur_la_page_qu_il_ouvre` :

```rust
    /// Les folios que la source publie sous `<mesures>`, un par repère, dans l'ordre.
    ///
    /// C'est exactement ce que la table affiche : elle lit les mêmes repères par la même
    /// requête, et rend le même `.location().page()`. Mesurer ici, plutôt que de lire la
    /// table rendue, évite de reconnaître des chiffres dans un PNG pour vérifier une
    /// valeur que Typst sait dire.
    fn folios_des_reperes(typst: &Typst, dossier: &Path, nom: &str, mut s: String) -> Vec<f64> {
        s.push_str(
            "\n#context [#metadata(query(<ozalid-tdm>).enumerate().fold((:), (d, it) => \
             d + ((str(it.at(0))): it.at(1).location().page())))<mesures>]\n",
        );
        let chemin = dossier.join(format!("{nom}.typ"));
        std::fs::write(&chemin, &s).expect("source non écrite");
        let folios = typst.mesures(&chemin).expect("mesures refusées");
        (0..folios.len())
            .map(|i| {
                *folios
                    .get(&i.to_string())
                    .unwrap_or_else(|| panic!("aucun repère au rang {i} : {folios:?}"))
            })
            .collect()
    }

    /// **La preuve du lot.** La table se compte elle-même dans les folios qu'elle
    /// affiche.
    ///
    /// C'est toute la mécanique de la spec § 2.3, et elle ne se raisonne pas : insérer
    /// une table décale les pièces qui la suivent, donc les folios qu'elle vient
    /// d'annoncer. Si Typst ne résolvait pas cette auto-référence en une invocation, la
    /// table renverrait le lecteur deux pages trop tôt — sur toutes les entrées, sans
    /// qu'aucun compte de pages ni aucun rendu ne le signale.
    ///
    /// L'écart entre les deux compositions est vérifié **constant**, et non figé à une
    /// valeur : c'est l'intention exacte — la table pousse tout le livre du même nombre
    /// de pages, celui qu'elle occupe elle-même, blanche de parité comprise. Un écart
    /// qui varierait d'une pièce à l'autre dirait que les folios ont été relevés avant
    /// l'insertion, ce que la spec écarte comme « deux passes côté Rust ».
    ///
    /// Les deux positions sont exercées : en fin, la table n'a rien à décaler et l'écart
    /// doit être **nul** sur les pièces, qui la précèdent toutes.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn la_table_se_compte_elle_meme_dans_les_folios_qu_elle_affiche() {
        let typst = typst_de_test();
        let dossier = tempfile::tempdir().expect("répertoire de travail");
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let r = Reglage {
            gouttiere: pr.gouttieres[0].2,
            blanche: false,
        };
        let pieces = pieces_des_quatre_sortes();
        let compose = |table: Table| {
            source(
                &livre(),
                &Interieur {
                    table,
                    ..Interieur::default()
                },
                pr,
                &r,
                &pieces,
                None,
            )
        };

        let sans = compose(Table::Absente);
        let n_sans = pages_de(&typst, dossier.path(), "sans", &sans);
        let f_sans = folios_des_reperes(&typst, dossier.path(), "sans-m", sans);
        assert_eq!(
            f_sans.len(),
            pieces.len(),
            "les repères du livre nu ne sont pas au complet : {f_sans:?}"
        );

        let en_tete = compose(Table::EnTete);
        let n_en_tete = pages_de(&typst, dossier.path(), "tete", &en_tete);
        let f_en_tete = folios_des_reperes(&typst, dossier.path(), "tete-m", en_tete);
        assert_eq!(
            f_en_tete.len(),
            pieces.len(),
            "la table s'est ajoutée aux repères, ou en a perdu : {f_en_tete:?}"
        );

        let ecarts: Vec<f64> = f_en_tete
            .iter()
            .zip(f_sans.iter())
            .map(|(a, s)| a - s)
            .collect();
        let decalage = ecarts[0];
        assert!(
            decalage >= 2.0,
            "la table en tête n'a poussé le livre que de {decalage} page(s) : \
             elle ne s'imprime pas, ou pas en belle page"
        );
        assert!(
            ecarts.iter().all(|e| *e == decalage),
            "la table n'a pas décalé toutes les pièces du même nombre de pages : \
             {ecarts:?} — les folios ont été relevés avant son insertion"
        );
        assert_eq!(
            f64::from(n_en_tete) - f64::from(n_sans),
            decalage,
            "le livre n'a pas grossi de ce dont la table a décalé les pièces : \
             {n_en_tete} pages contre {n_sans}"
        );

        let en_fin = compose(Table::EnFin);
        let n_en_fin = pages_de(&typst, dossier.path(), "fin", &en_fin);
        let f_en_fin = folios_des_reperes(&typst, dossier.path(), "fin-m", en_fin);
        assert_eq!(
            f_en_fin, f_sans,
            "une table en fin a déplacé des pièces qui la précèdent toutes"
        );
        assert!(
            n_en_fin > n_sans,
            "une table en fin n'a ajouté aucune page : {n_en_fin} contre {n_sans}"
        );
        // **La belle page se prouve ici, et nulle part ailleurs en composant.** En tête,
        // le copyright rend toujours la main sur une impaire : un saut simple donnerait
        // le même livre, et le saut de parité y est une garde sans effet observable. En
        // fin, la parité dépend de la longueur des annexes — le livre nu s'arrête sur
        // une impaire, la table doit donc sauter la paire qui suit.
        assert_eq!(
            n_sans % 2,
            1,
            "ce test ne prouve la belle page que sur un livre dont la pagination nue est \
             impaire ; elle vaut {n_sans}. Allonger le manuscrit de test d'une page — ne \
             pas retirer cette garde, elle est ce qui empêche le test de devenir muet."
        );
        let ouverture = n_sans + 2;
        assert!(
            n_en_fin >= ouverture,
            "la table en fin s'est ouverte au verso : le livre fait {n_en_fin} pages, \
             elle devait s'ouvrir en {ouverture}"
        );
    }

    /// Une table longue **converge quand même** : elle occupe plusieurs pages, et les
    /// folios qu'elle affiche tiennent compte de toutes.
    ///
    /// Le cas court d'à côté ne le prouve pas : une table d'une page ajoute deux pages
    /// (elle et sa blanche) quel que soit le nombre de tours que Typst fait. Ici la
    /// table déborde, ce qui la fait grossir de son propre débordement — c'est le point
    /// fixe que la spec § 2.3 confie au moteur plutôt qu'à une boucle Rust.
    ///
    /// Quarante chapitres d'une page : la table en occupe deux, et les folios doivent
    /// rester **consécutifs** à partir de sa sortie. Un point fixe manqué se voit à un
    /// folio de trop bas d'exactement une page sur toute la suite.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn une_table_qui_deborde_sur_deux_pages_converge() {
        let typst = typst_de_test();
        let dossier = tempfile::tempdir().expect("répertoire de travail");
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let r = Reglage {
            gouttiere: pr.gouttieres[0].2,
            blanche: false,
        };
        let pieces = manuscrit_long();
        let s = source(
            &livre(),
            &Interieur {
                table: Table::EnTete,
                ..Interieur::default()
            },
            pr,
            &r,
            &pieces,
            None,
        );
        let folios = folios_des_reperes(&typst, dossier.path(), "longue", s);
        assert_eq!(folios.len(), pieces.len());
        let premier = folios[0];
        assert!(
            premier >= 7.0,
            "la table n'a pas repoussé le premier chapitre : il ouvre en {premier}"
        );
        let attendus: Vec<f64> = (0..pieces.len()).map(|i| premier + i as f64).collect();
        assert_eq!(
            folios, attendus,
            "les folios ne sont plus consécutifs : la table ne s'est pas comptée en entier"
        );
    }
```

- [ ] **Étape 2 : lancer les deux tests**

```bash
cd src-tauri && cargo test --lib -- --ignored la_table_se_compte une_table_qui_deborde 2>&1 | tail -30
```

Attendu : les deux au vert. **En cas de rouge sur le premier `assert!(premier >= 7.0)`**, la
table de quarante entrées tient sur une seule page dans ce gabarit : ce n'est pas un défaut,
mais le test ne prouve alors plus rien — allonger les titres de `manuscrit_long()` n'est pas
une option (d'autres tests s'en servent), **ajouter une fonction locale** de soixante
chapitres l'est, sur le modèle vérifié le 29/08.

- [ ] **Étape 3 : voir l'écart constant rouge par mutation**

Ce que l'écart constant protège, c'est que la table pousse **tout** le livre — et non la
seule partie qui la suit. La faute se simule en la posant au milieu des liminaires : dans
`liminaires()`, déplacer le bloc `if int.table == Table::EnTete { … }` **après** la boucle
`for p in pieces`, si bien que la préface la précède et ne bouge plus.

```bash
cd src-tauri && cargo test --lib -- --ignored la_table_se_compte 2>&1 | tail -20
cd src-tauri && cargo test --lib la_table_en_tete_se_compose 2>&1 | tail -10
```

Attendu : **échec du test composé** sur « la table n'a pas décalé toutes les pièces du même
nombre de pages » — la préface garde son folio, les quatre autres pièces reculent —, et
échec du test unitaire de position. **Remettre le bloc à sa place**, relancer, vérifier le
vert des deux.

- [ ] **Étape 4 : voir la belle page composée rouge par mutation**

Reposer la mutation de la tâche 2 — dans `table_matieres`, `String::from("#pagebreak()\n")`
à la place du saut de parité — et la lancer cette fois sur le test composé :

```bash
cd src-tauri && cargo test --lib -- --ignored la_table_se_compte 2>&1 | tail -20
```

Attendu : **échec** sur « la table en fin s'est ouverte au verso ». C'est la même faute que
la tâche 2 attrape dans le texte, prouvée ici dans la pagination : le livre fait une page de
moins parce que la table a commencé au verso. **Rétablir le saut de parité**, relancer,
vérifier le vert.

Si l'une de ces deux mutations restait verte, **ne pas passer outre** : c'est que ces tests
ne protègent pas ce qu'ils prétendent, et il faut le dire dans le compte rendu de tâche
plutôt que cocher l'étape.

- [ ] **Étape 5 : la suite complète, ignorés compris**

```bash
cd src-tauri && cargo test 2>&1 | tail -5
cd src-tauri && cargo test -- --ignored 2>&1 | tail -10
```

Attendu : tout au vert. En particulier `chaque_repere_est_situe_sur_la_page_qu_il_ouvre`
(folios `[5, 7, 9, 10, 11]`) et `les_reperes_n_occupent_aucune_place_et_ne_se_voient_nulle_part`
**restent verts sans être touchés** : ils composent avec `Interieur::default()`, donc sans
table. **Un rouge sur l'un des deux est une régression de ce lot, pas une valeur à relever.**

- [ ] **Étape 6 : commit**

```bash
cd src-tauri && cargo fmt && cargo fmt --check
cd /Users/jean-paulgavini/Documents/Dev/OzalidStudio
git add src-tauri/src/interieur.rs
git commit -m "La table se compte elle-même dans ses folios, composition à l'appui"
```

---

### Tâche 4 : L'onglet Livre allume la table

**Fichiers :**
- Modifier : `src/index.html` — le bloc `Intérieur`, sous le sélecteur de police
- Modifier : `src/app.js` — `TAILLES`, `majInterieur`, l'affichage, l'écouteur
- Test : `tests/epreuve.test.js` — c'est là que vivent les tests des réglages d'intérieur

**Interfaces :**
- Consomme : la commande `interieur_modifier` et le champ `interieur` du projet, qui portent
  désormais `table` (chaîne kebab) et `entree_table` (nombre) — tâche 1
- Produit : rien pour le Rust. Le front n'invente aucun nom : ceux de gauche dans `TAILLES`
  sont ceux de `Interieur`, serde les lit tels quels.

- [ ] **Étape 1 : écrire les tests, qui échouent sur un élément absent**

Dans `tests/epreuve.test.js`, étendre d'abord la fixture — elle se dit complète, et une
fixture qui tairait deux champs ferait paraître des champs vides là où l'application en
montre douze :

```js
  // Les douze tailles et le réglage de table, tels que le Rust les sert : `Interieur`
  // n'en tait aucun, et une fixture qui en oublierait ferait paraître un champ vide là
  // où l'application en montre toujours douze.
  interieur: {
    police: 'Alegreya',
    table: 'absente',
    corps: 9.5, faux_titre: 11, page_titre_auteur: 10.5, page_titre_titre: 15,
    page_titre_genre: 10, copyright: 8, dedicace: 9.5, numero: 13,
    titre_section: 10, ouverture_piece: 10, entree_table: 9, folio: 8,
  },
```

puis ajouter, à la suite de `changer une taille enregistre le réglage dans le projet` :

```js
/**
 * Allumer la table part au Rust dans la forme qu'il attend — `"en-tete"`, la valeur de
 * l'option, et non un libellé d'interface. Une chaîne que serde ne connaît pas
 * échouerait à mi-chemin, sur un message qui ne nomme aucun champ.
 *
 * Le réglage voyage avec les douze tailles, dans le même envoi : `interieur_modifier`
 * reçoit un `Interieur` entier, jamais un champ isolé.
 */
test('allumer la table enregistre le réglage dans le projet', async () => {
  let recu = null;
  const { els } = await charge({
    invoke: faux([LULU], {
      projet_importer: PROJET,
      interieur_modifier: (args) => {
        recu = args.interieur;
        return { ...PROJET, interieur: args.interieur };
      },
    }),
    open: async () => '/dev/ozalid/build/LHC/livre.toml',
  });
  await els.get('btImporter').declenche('click');
  els.get('inTable').value = 'en-tete';
  await els.get('inTable').declenche('change');
  assert.deepStrictEqual(
    { ...recu },
    { ...PROJET.interieur, table: 'en-tete' },
    'le réglage part seul, les douze tailles inchangées',
  );
  assert.strictEqual(els.get('inTable').value, 'en-tete');
});

/**
 * La taille d'une entrée de table est une taille comme les onze autres : elle part en
 * nombre, et elle revient du projet. Le sélecteur d'à côté ne la remplace pas — une
 * table absente garde sa taille, qui reparaîtra telle quelle si on l'allume.
 */
test('la taille d\'une entrée de table fait l\'aller-retour', async () => {
  let recu = null;
  const { els } = await charge({
    invoke: faux([LULU], {
      projet_importer: PROJET,
      interieur_modifier: (args) => {
        recu = args.interieur;
        return { ...PROJET, interieur: args.interieur };
      },
    }),
    open: async () => '/dev/ozalid/build/LHC/livre.toml',
  });
  await els.get('btImporter').declenche('click');
  els.get('inTailleTable').value = '8.5';
  await els.get('inTailleTable').declenche('change');
  assert.deepStrictEqual({ ...recu }, { ...PROJET.interieur, entree_table: 8.5 });
  assert.strictEqual(typeof recu.entree_table, 'number');
});
```

- [ ] **Étape 2 : lancer les tests, les voir rouges**

```bash
cd /Users/jean-paulgavini/Documents/Dev/OzalidStudio && node --test tests/epreuve.test.js 2>&1 | tail -25
```

Attendu : **échec** — `inTable` et `inTailleTable` n'existent pas, et
`changer une taille enregistre le réglage` rougit aussi, la fixture portant désormais deux
champs que `majInterieur` n'envoie pas.

- [ ] **Étape 3 : le HTML**

Dans `src/index.html`, bloc `<h2>Intérieur</h2>`, **entre le sélecteur de police et la note
sur le repli des polices** — le réglage se règle près de la police, pour la même raison
qu'elle (spec § 2.1) :

```html
      <label><span>Table des matières</span>
        <select id="inTable">
          <option value="absente">Absente</option>
          <option value="en-tete">En tête</option>
          <option value="en-fin">En fin</option>
        </select></label>
      <p class="note">Une table reprend parties, chapitres et pièces, avec leur folio.
        <strong>L'allumer ajoute des pages, donc change le dos</strong> : la composition
        repart, et la planche reste sans dos le temps qu'elle tourne. En tête, elle vient
        avant la préface ; en fin, après les annexes.</p>
```

et, dans la liste des tailles, **entre « Titre de préface ou de postface » et « Folio »** :

```html
      <label><span>Entrée de table des matières</span>
        <input type="number" id="inTailleTable" min="4" max="48" step="0.5"></label>
```

Corriger enfin, dans le commentaire au-dessus de la liste, « Les onze tailles » en « Les
douze tailles », et dans la note visible « ce sont onze leviers » en « douze leviers ».

- [ ] **Étape 4 : le JavaScript**

Dans `src/app.js`, `TAILLES`, entre `ouverture_piece` et `folio` :

```js
  entree_table: 'inTailleTable',
```

Corriger le doc-comment de `TAILLES` : « Les onze tailles » → « Les douze tailles ».

Dans `majInterieur`, la première ligne :

```js
  const interieur = { police: $('inPoliceInterieur').value, table: $('inTable').value };
```

Dans l'affichage, à côté de `$('inPoliceInterieur').value = p.interieur.police;` :

```js
  $('inTable').value = p.interieur.table;
```

Et l'écouteur, à côté de celui de la police :

```js
$('inTable').addEventListener('change', majInterieur);
```

- [ ] **Étape 5 : lancer les tests du front**

```bash
cd /Users/jean-paulgavini/Documents/Dev/OzalidStudio && node --test tests/*.test.js 2>&1 | tail -8
```

Attendu : tout au vert, les 291 d'avant et les deux neufs.

- [ ] **Étape 6 : voir l'aller-retour rouge par mutation**

Retirer `$('inTable').value = p.interieur.table;` de l'affichage — la panne exacte que ce
couple de tests existe pour attraper : un réglage qui part bien mais se repose à l'ancienne
au premier affichage, symptôme indiscernable d'un refus du Rust.

```bash
cd /Users/jean-paulgavini/Documents/Dev/OzalidStudio && node --test tests/epreuve.test.js 2>&1 | tail -15
```

Attendu : **échec** sur `assert.strictEqual(els.get('inTable').value, 'en-tete')`.
**Rétablir la ligne**, relancer, vérifier le vert.

- [ ] **Étape 7 : commit**

```bash
cd /Users/jean-paulgavini/Documents/Dev/OzalidStudio
git add src/index.html src/app.js tests/epreuve.test.js
git commit -m "L'onglet Livre allume la table et règle ses entrées"
```

---

### Tâche 5 : Le témoin garde la pagination, l'ebook suit, le README le dit

**Fichiers :**
- Modifier : `src-tauri/examples/temoin.rs` — `TEMOINS` et sa boucle
- Modifier : `README.md` — section « 1 · Livre »
- Test : `src-tauri/src/interieur.rs`, module `mod tests` (l'ebook)

**Interfaces :**
- Consomme : `Table`, `Interieur` (tâche 1), `Projet::modifier_interieur` (`projet.rs`)
- Produit : une troisième pagination de référence, figée dans `TEMOINS`

- [ ] **Étape 1 : le test de l'ebook**

Le PDF ebook suit le réglage sans une ligne de code — `source_ebook` appelle le même
`assemble`. Ce qui n'a pas de code a d'autant plus besoin d'un test : rien n'empêcherait un
lot futur de passer un `Interieur::default()` à l'ebook « pour ne pas alourdir la liseuse ».

À ajouter dans `mod tests`, à la suite de `l_ebook_compose_sans_gouttiere_ni_blanche_de_parite` :

```rust
    /// **Le PDF ebook porte la table** : c'est le même livre, sans son imposition
    /// (spec § 3). L'archive EPUB, elle, garde sa table de navigation native — elle ne
    /// se pagine pas, et `epub.rs` ne connaît pas `Interieur`.
    #[test]
    fn le_pdf_ebook_suit_le_reglage_de_table() {
        let pr = provider("lulu").expect("gabarit lulu");
        let pieces = pieces_des_quatre_sortes();
        let sans = source_ebook(
            &livre(),
            &Interieur::default(),
            pr,
            &pieces,
            "#page[couverture]\n",
        );
        assert!(!sans.contains("Table des matières"), "table non demandée\n{sans}");
        let avec = source_ebook(
            &livre(),
            &Interieur {
                table: Table::EnFin,
                ..Interieur::default()
            },
            pr,
            &pieces,
            "#page[couverture]\n",
        );
        assert!(
            avec.contains("Table des matières") && avec.contains(&format!("query(<{TDM}>)")),
            "l'ebook n'a pas composé la table\n{avec}"
        );
    }
```

```bash
cd src-tauri && cargo test --lib le_pdf_ebook_suit 2>&1 | tail -10
```

Attendu : **vert du premier coup** — le code est déjà là. Le voir rouge demande la mutation
de l'étape suivante.

- [ ] **Étape 2 : voir le test de l'ebook rouge par mutation**

Dans `source_ebook`, composer avec un intérieur neuf plutôt qu'avec celui du projet :

```rust
    assemble(&ctx, &Interieur::default(), pr, &r, pieces, None, Some(couverture))
```

```bash
cd src-tauri && cargo test --lib le_pdf_ebook_suit 2>&1 | tail -10
```

Attendu : **échec** sur « l'ebook n'a pas composé la table ». **Rétablir `int`**, relancer,
vérifier le vert. (Ce rouge-là prouve du même coup que le test attrape la régression qu'il
vise, et pas seulement la présence d'une chaîne.)

- [ ] **Étape 3 : le troisième témoin**

Dans `src-tauri/examples/temoin.rs`, ajouter l'import et étendre la table :

```rust
use ozalid_lib::interieur::{Interieur, Table};
```

```rust
/// Les fabrications composées, la table qu'elles portent, et la pagination attendue de
/// chacune.
///
/// Chaque pagination est **relevée**, sur macOS avec Typst 0.15.1 et EB Garamond, au
/// corps et à l'interligne que `interieur` fixe pour tout gabarit. Elle dépend de chacun
/// de ces éléments : la déplacer est un acte délibéré, à revalider sur un livre réel —
/// jamais un ajustement pour faire passer l'intégration continue.
///
/// **Le troisième témoin allume la table**, sur le même gabarit que le premier : c'est
/// la seule façon de garder la pagination d'un livre qui en porte une. Sans lui, la
/// mesure ne vivrait que dans un document, et un document ne se relance pas. Les deux
/// premiers restent table absente — ils gardent, eux, la promesse que ce lot n'a rien
/// changé aux livres qui n'ont rien demandé.
const TEMOINS: &[(&str, &str, &str, &str, Table, u32)] = &[
    ("bod", "135x215", "broche", "creme-90", Table::Absente, 98),
    ("bod", "120x190", "broche", "photo-brillant-130", Table::Absente, 118),
    ("bod", "135x215", "broche", "creme-90", Table::EnTete, 0),
];
```

Dans la boucle, régler la table avant chaque composition et écrire dans un répertoire propre
à chaque témoin — deux fabrications d'un même gabarit partagent sinon leur intérieur, et la
seconde écraserait le PDF de la première :

```rust
    for &(pod, format, reliure, papier, table, attendues) in TEMOINS {
        projet.modifier_interieur(Interieur {
            table,
            ..Default::default()
        });
        let ou = sortie.join(match table {
            Table::Absente => "sans-table",
            Table::EnTete => "table-en-tete",
            Table::EnFin => "table-en-fin",
        });
        match compose(&projet, &typst, &ou, (pod, format, reliure, papier)) {
            Ok(pages) if pages != attendues => ecarts.push(format!(
                "{pod}-{format}-{reliure}-{papier} ({table:?}) : {pages} pages, \
                 {attendues} attendues"
            )),
            Ok(_) => {}
            Err(e) => ecarts.push(format!("{pod}-{format}-{reliure}-{papier} : {e}")),
        }
    }
```

`Table` doit être affichable par `{table:?}` : elle dérive `Debug` (tâche 1).

- [ ] **Étape 4 : relever la pagination du troisième témoin**

Le `0` de la table est un rouge délibéré : il fait dire au témoin la valeur réelle.

```bash
cd src-tauri && cargo run --example temoin 2>&1 | tail -20
```

Attendu : les deux premiers à **98** et **118**, et un écart annoncé sur le troisième —
« *bod-135x215-broche-creme-90 (EnTete) : N pages, 0 attendues* ». **Remplacer le `0` par
`N`**, relancer, vérifier que tout passe.

Deux contrôles sur `N` avant de le figer :
- il vaut **98 + 2 au moins** : une table de Candide (trente chapitres) tient sur une page,
  qui appelle sa blanche ;
- il est **pair** — le témoin passe par `converge`, qui ajoute la blanche de parité.

Si `N` valait 98, la table ne s'imprime pas : ne pas figer, chercher la cause.

- [ ] **Étape 5 : le README**

Dans `README.md`, section « 1 · Livre », après le point sur la police de l'intérieur :

```markdown
- **La table des matières** est un réglage à trois états — absente, en tête, en fin —,
  **éteinte par défaut**. Elle reprend parties, chapitres et pièces sur deux rangs, avec
  l'intitulé que leur page d'ouverture imprime et le folio où elles s'ouvrent. En tête,
  elle vient avant la préface ; en fin, après les annexes. **L'allumer ajoute des pages,
  donc change le dos** — la composition repart d'elle-même, et le pied dit où elle en est.
  Le PDF ebook la porte aussi ; l'EPUB, lui, garde sa table de navigation native.
```

- [ ] **Étape 6 : la suite complète**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets 2>&1 | grep -c "interieur.rs\|temoin.rs"
cd src-tauri && cargo test 2>&1 | tail -5
cd src-tauri && cargo test -- --ignored 2>&1 | tail -10
cd /Users/jean-paulgavini/Documents/Dev/OzalidStudio && node --test tests/*.test.js 2>&1 | tail -5
cd src-tauri && cargo run --example temoin 2>&1 | tail -20
```

Attendu : `grep -c` rend `0`, tout au vert, témoin à **98 / 118 / N**.

- [ ] **Étape 7 : commit**

```bash
cd /Users/jean-paulgavini/Documents/Dev/OzalidStudio
git add src-tauri/examples/temoin.rs src-tauri/src/interieur.rs README.md
git commit -m "Le témoin garde la pagination sous table, l'ebook la suit, le README la dit"
```

---

## Ce que ce lot ne fait pas

- **Il ne distingue pas une pièce liminaire d'une annexe** dans la table : les deux
  paraissent au second rang, comme un chapitre. C'est la dette relevée au lot 2 ; un
  quatrième champ dans le repère serait un autre lot.
- **Il ne touche pas `epub.rs`** : l'archive garde sa table de navigation native.
- **Il ne touche pas `epreuve.rs`** : l'épreuve se relit, elle ne se feuillette pas.
- **Il ne change pas le message de refus de pagination** (décision 8).
- **Il ne corrige pas la baseline clippy** de `police.rs:123`.

## Vérification de fin de lot

- `cargo fmt --check` propre
- `cargo clippy --all-targets -- -D warnings` : aucun avertissement citant `interieur.rs` ni
  `temoin.rs` (l'échec sur `police.rs:123` est la baseline, voir contraintes globales)
- `cargo test` au vert, et `cargo test -- --ignored` au vert
- `node --test tests/*.test.js` au vert
- `cargo run --example temoin` : **98 pages / dos 7,21 mm**, **118 pages**, et le troisième
  témoin à sa valeur relevée
- Les **huit mutations** ont été posées, lancées, **vues rouges**, puis retirées : une en
  tâche 1 (la taille hors de `tailles()`), trois en tâche 2 (le saut simple, l'étiquette sur
  la table, le folio retouché), deux en tâche 3 (la table après la préface, le saut simple
  prouvé cette fois dans la pagination), une en tâche 4 (l'affichage retiré), une en tâche 5
  (l'ebook composé sur un intérieur neuf)

## Ce qu'aucun test ne verra, et qui se regarde

- `cargo run --example temoin /tmp/tdm`, puis **la table de Candide ouverte à l'œil** dans le
  PDF de `table-en-tete/` : les points de conduite, l'alignement des folios à droite, le
  blanc au-dessus des lignes.
- **La même chose sur un manuscrit à parties**, en tête puis en fin : c'est le seul cas où
  les deux rangs se voient, et où l'indentation conditionnelle a un sens.
- **La blanche de parité**, quand la table finit sur une impaire.
- **Dans la fenêtre** (`cargo tauri dev`) : allumer la table et voir le pied passer au dos
  périmé, puis la composition repartir seule. `modifier_interieur` oublie les mesures — c'est
  le comportement attendu, jamais vérifié à l'écran depuis le lot 1.
