# Les mentions légales qui se citent, et la table des matières

Date : 2026-08-28
Statut : validé (brainstorming)

## Objectif

Deux manques du **livre édité**, relevés le 28/08. Ils tiennent au même endroit du volume —
ses pages liminaires — et se règlent par deux mécaniques indépendantes.

1. **L'ISBN est saisi une fois et retapé une seconde.** `projet.rs` le porte depuis le
   chantier précédent, et `couverture.rs` le compose en code-barres sur la 4ème. Mais
   `gabarit.rs` ne connaît que six jetons, tous branchés sur un champ de `Livre` : le pavé
   de copyright de la page 4 ne peut pas citer l'ISBN. L'auteur le recopie donc à la main,
   à côté du code-barres qui le dessine. C'est exactement le geste que ce projet combat sur
   le dos. Le dépôt légal et le nom de l'imprimeur — deux mentions que la loi française
   réclame — sont dans le même cas, alors que l'imprimeur est déjà connu du livrable.
2. **Le livre n'a pas de table des matières.** `epub.rs` en compose une pour la liseuse ;
   l'intérieur imprimé n'en a aucune. Dès qu'un livre porte des parties, une préface ou une
   postface, elle manque.

Le premier volet **n'ajoute pas une ligne** au livre : le pavé de copyright est déjà là. Le
second en ajoute des pages, et donc change le dos — délibérément, et seulement quand
l'auteur allume le réglage.

## Décisions de cadrage (brainstorming du 28/08)

- **La table est un réglage à trois états** — absente, en tête, en fin — **éteinte par
  défaut**. Même parti que la collection sur le dos : allumée d'office, elle ajouterait des
  pages à tous les livres existants, donc modifierait leur dos sans que personne l'ait
  demandé.
- **Elle reprend parties, chapitres et pièces**, sur deux rangs. Une préface qui a sa page
  d'ouverture et n'apparaît pas dans la table serait un défaut, pas une simplification.
- **Les mentions passent par des jetons, imprimeur compris.** Le pavé de copyright reste un
  champ libre : c'est un acquis, la page 4 n'a pas la même forme d'un éditeur à l'autre. Une
  page typée aurait remplacé cette liberté au lieu de l'étendre.
- **Le PDF ebook suit le réglage de table.** C'est le même livre, sans son imposition.
  L'EPUB, lui, garde sa table de navigation native, qui ne se pagine pas.
- **Un relevé a corrigé le cadrage en séance** : on croyait que `%IMPRIMEUR%` romprait le
  partage d'intérieur entre livrables. Il n'en est rien — `catalogue.rs:915` donne
  `cle_gabarit() = pod-format-reliure`, le POD y est déjà. Le partage ne joue qu'entre
  papiers et finitions d'un même imprimeur, où le nom est constant.

## 1. Les mentions légales par jetons

### 1.1 Aucune page nouvelle

`interieur.rs:574-587` cale le pavé de copyright au bas de la justification, sur le verso de
la page de titre. C'est précisément la place française des mentions légales. Le volet ne
déplace rien et n'insère rien : **le témoin doit rester inchangé** — 98 pages, dos 7,21 mm.
C'est sa preuve, pas une observation d'accompagnement.

### 1.2 Trois jetons de plus

`gabarit.rs:19` en compte six. Trois s'y ajoutent :

| Jeton | Ce qu'il rend |
|---|---|
| `%ISBN%` | `livre.isbn`, tel qu'il est saisi avec ses tirets — la forme qui s'imprime déjà en clair au-dessus du code-barres |
| `%DEPOT_LEGAL%` | un champ neuf `Livre::depot_legal`, texte libre |
| `%IMPRIMEUR%` | `Provider::pod_nom` du livrable — « BoD », jamais `bod` |

`depot_legal` est **saisi, jamais dérivé de l'année courante** : la date du dépôt est celle
d'un acte administratif, pas celle de la compilation. Un défaut calculé se lirait comme une
mesure — le même argument que celui qui interdit de préremplir un fond perdu relevé.

Il se saisit dans l'onglet Livre, voisin de l'ISBN, et naît vide : un tirage privé n'en a
pas, et le pavé saute les lignes vides.

### 1.3 Le contexte de substitution

Les six jetons actuels pointent tous vers un champ de `Livre`, ce dont vit la signature
`substituer(&str, &Livre)`. L'imprimeur, lui, vient de la cible.

`substituer` prend donc un contexte :

```rust
pub struct Contexte<'a> {
    pub livre: &'a Livre,
    /// Le nom de l'imprimeur, quand la composition en vise un. `None` pour l'ebook.
    pub imprimeur: Option<&'a str>,
}
```

**Une seule porte, une seule table.** L'alternative — garder `substituer` et ajouter une
seconde fonction pour l'imprimeur — ouvrait deux chemins de substitution, dont le second
serait tôt ou tard oublié sur un champ libre. `Provider::pod_nom` existe déjà et pour cette
raison-là : il a été mis sur le `Provider` pour que la fiche de téléversement n'ait rien à
retraduire.

### 1.4 Dans l'ebook, l'imprimeur s'efface

`imprimeur: None` rend la **chaîne vide**, pas le jeton littéral. Un EPUB n'est pas imprimé,
et un `%IMPRIMEUR%` resté en toutes lettres dans le pavé serait une faute visible du lecteur.
La ligne devenue vide disparaît, comme le pied de la 4ème saute déjà les siennes.

C'est la mutation à poser en premier sur ce volet : laisser le jeton littéral dans l'ebook
doit faire rougir un test nommé.

## 2. La table des matières

### 2.1 Le réglage

```rust
pub enum Table {
    #[default]
    Absente,
    EnTete,
    EnFin,
}
```

**Le réglage vit dans `Interieur`**, à côté de la police, et pour la même raison qu'elle :
c'est un choix de composition qui change la pagination, donc le dos. Il n'appartient pas à
`Livre`, qui porte l'identité du livre et non la façon de le composer. La struct portant déjà
`#[serde(default)]`, un `.ozalid` ancien se relit sans table.

Il se règle dans l'onglet Livre, près de la police — dans le sillage du commit qui vient d'y
faire descendre les onze tailles. Une **douzième taille** rejoint la même struct sous le nom
`table`, avec sa ligne dans `verifie()` et dans la liste de `interieur.rs:130`.

### 2.2 Ce qu'une ligne dit

`manuscrit.rs:56` donne les quatre sortes. `Partie` tient le premier rang ; `Chapitre`,
`Liminaire` et `Annexe` le second, indentés.

Chaque ligne reprend **l'intitulé que la page d'ouverture imprime** — numéro et titre pour un
chapitre, romain et titre pour une partie, titre seul pour une pièce —, des points de
conduite, le folio. Une pièce sans titre (`## 7`, cas admis par le format) donne son numéro
seul : la table ne fabrique aucun libellé que le livre n'imprime pas.

### 2.3 La mécanique, et pourquoi celle-là

Un `#metadata(…) <ozalid-tdm>` est posé à l'ouverture de chaque pièce. La table se compose
en `context query(<ozalid-tdm>)`, chaque entrée rendant son folio par `.location().page()`.

**Typst résout seul l'auto-référence** : la table occupe des pages, et les folios qu'elle
affiche en tiennent compte, en une seule invocation. Le procédé est acquis dans ce dépôt —
`epreuve.rs:57-74` emploie déjà `context`, `query` et `counter(page)` sur la version épinglée
0.15.1.

Deux voies écartées :

- **`outline()` natif** est hors d'atteinte. L'intérieur compose ses ouvertures à la main
  (`interieur.rs:644`) sans un seul `heading`, et en poser réécrirait le flux : un gabarit de
  titre, des espacements avant et après, donc un risque de pagination modifiée pour un
  bénéfice nul.
- **Deux passes côté Rust** — relever les folios, réécrire la source — obligeraient à itérer
  jusqu'au point fixe comme `converge` le fait pour la gouttière, puisque insérer la table
  décale les folios qu'on vient de relever. Deux compositions de plus pour ce que le moteur
  donne seul.

### 2.4 La neutralité se prouve, elle ne s'affirme pas

**Les repères sont posés toujours**, table allumée ou non. C'est ce qui permet au témoin de
rester à 98 pages / dos 7,21 mm après le lot qui les pose, et donc de prouver qu'un
`metadata` n'occupe aucune place.

Ne les poser que sous réglage allumé aurait rendu cette preuve impossible : allumer la table
changerait alors deux choses à la fois, et un écart de pagination n'aurait plus de coupable
identifiable.

### 2.5 Belle page, folio, dos

La table **s'ouvre en page impaire**, et une blanche la suit si elle finit sur une impaire.
Elle reste **hors folio** dans les deux positions : en tête elle rejoint la série des
liminaires, en fin la zone que `interieur.rs:401` tient déjà hors du folio avec les annexes.

Allumer la table change la pagination, donc le dos. C'est le comportement normal et rien n'est
à prévoir pour lui : le réglage périme la mesure comme un changement de police, et
l'application recompose seule.

## 3. Ce qui bouge ailleurs

- **`epub.rs`** : le PDF ebook suit le réglage ; l'archive EPUB garde sa table de navigation
  native, intouchée. Le pavé de copyright y passe par `Contexte { imprimeur: None }`.
- **`epreuve.rs`** : rien. L'épreuve se relit, elle ne se feuillette pas — sa source a son
  propre gabarit à `heading`, sans rapport avec celui de l'intérieur.
- **Le front** : deux saisies dans l'onglet Livre — le dépôt légal, le réglage de table —, et
  les trois jetons neufs dans l'aide déjà servie par `gabarit::jetons()`, jamais recopiés dans
  le HTML.
- **Le README** : la liste des jetons, le réglage de table, et la mention que le dos bouge
  quand on l'allume.
- **Compatibilité** : `depot_legal` naît avec son `#[serde(default)]`, comme `titre_page` avant
  lui ; le réglage de table hérite de celui que `Interieur` porte déjà sur sa struct.
  **`VERSION` ne bouge pas** — un `.ozalid` ancien s'ouvre sans table et sans dépôt légal, ce
  qui est son état exact.

## 4. Risques

- **Le `metadata` n'est pas aussi neutre qu'annoncé.** C'est le risque central, et il est
  mesuré dès le lot 2 : si le témoin bouge d'une page, le procédé est faux et la spec doit
  être rouverte avant d'écrire la moindre ligne de table.
- **Une table longue dans un livre court** peut ajouter plus de pages qu'on ne l'imagine, et
  déplacer le livre hors de la pagination admise par la reliure. Le contrôle de pagination
  existe déjà côté catalogue et refusera ; il faut vérifier que son message reste lisible dans
  ce cas-là, où la cause n'est pas le texte mais un réglage.
- **`%IMPRIMEUR%` dans un champ libre autre que le copyright** — la 4ème, le titre de la page
  de titre — se résoudra aussi. C'est cohérent et volontaire, mais à regarder : rien
  n'interdira d'écrire le nom de l'imprimeur sur la couverture.

## 5. Vérification

Sur chaque lot : `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`, `node --test tests/*.test.js`, et `cargo run --example temoin`.

- **Lots 1 et 2 : le témoin doit être inchangé** — 98 pages, dos 7,21 mm, et la seconde
  fabrication à 118 pages.
- **Lot 3 : le témoin bouge**, et sa mesure sous table allumée devient une référence neuve,
  écrite dans le plan du lot.
- **Pas de test qui n'ait été vu rouge.** Mutations à poser et à lancer, jamais raisonnées :
  `%IMPRIMEUR%` laissé littéral dans l'ebook ; `%ISBN%` branché sur un autre champ ; la table
  ouverte en page paire ; une `Partie` rendue au second rang ; un `depot_legal` prérempli à
  l'année courante.
- **Ce qu'aucun test ne verra, et qui se regarde** : `cargo run --example packager`, puis la
  page 4 composée, lue à l'œil ; la table sur un manuscrit à parties, en tête puis en fin ; la
  blanche de parité ; et dans la fenêtre, le pied qui passe au dos périmé quand on allume la
  table.
- **Les gardes du dépôt** : `touch src-tauri/src/lib.rs` après un changement de `src/` ;
  `cargo test -- --ignored` si `package.rs` bouge.

## 6. Les lots

1. **Les mentions se citent** — les trois jetons, le `Contexte`, `depot_legal`, la saisie dans
   l'onglet Livre, l'effacement dans l'ebook. Témoin inchangé.
2. **Les repères, sans la table** — les `metadata` labellisés à chaque ouverture, rien qui
   s'affiche. Témoin inchangé : c'est le lot dont la preuve est le seul livrable.
3. **La table** — le réglage à trois états, la composition par `query`, la belle page, la
   douzième taille, le PDF ebook qui suit.

## Hors périmètre

- **L'index** — une table des matières n'est pas un index, qui demanderait de marquer des
  entrées dans le manuscrit et donc d'élargir le sous-ensemble Markdown accepté.
- **Un ours complet** — nom et adresse de l'imprimeur, numéro d'impression : le pavé libre les
  accepte en texte, aucune donnée typée ne leur est ajoutée ici.
- **La table de l'EPUB** — elle existe, elle est native, elle ne se pagine pas.
- **L'épreuve de relecture** — elle ne prend pas la table.
