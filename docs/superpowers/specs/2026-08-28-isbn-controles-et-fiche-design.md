# Le code-barres, les contrôles avant envoi, et la fiche de téléversement

Date : 2026-08-28
Statut : validé (brainstorming)

## Objectif

Trois manques relevés le 28/08, qui ont en commun d'être **les derniers pas de la chaîne** —
ceux que l'application laisse aujourd'hui faire à la main, hors de sa vue.

1. **La 4ème réserve une zone ISBN et n'y met rien.** `couverture.rs` y pose un
   `rect(fill: "#ffffff")`, et `Livre` ne porte aucun ISBN. L'auteur doit donc fabriquer son
   code-barres ailleurs et le recoller dans un fichier que l'application ne contrôle plus.
   Sans code-barres, le livre n'entre pas en librairie.
2. **Rien ne dit qu'une image est trop pauvre pour l'impression, ni qu'un dos est trop mince
   pour porter du texte.** Les deux se découvrent sur l'exemplaire reçu. Les deux se
   calculent.
3. **Le compte rendu qu'on emporte chez l'imprimeur vit à l'écran.** Le COOKBOOK décrit les
   réglages à saisir ; le dossier livré, lui, ne porte que des PDF muets.

Ces trois chantiers ne touchent ni la pagination, ni la géométrie de la planche, ni le
catalogue existant : le témoin ne doit pas bouger d'une page.

## Décisions de cadrage (brainstorming du 28/08)

- **Le code-barres se dessine, il ne se colle pas.** C'est la devise du projet appliquée à
  la 4ème : le dos découle de la pagination sans être ressaisi, le code-barres découlera de
  l'ISBN sans être recollé.
- **Un ISBN qui ne vérifie pas sa clé fait refuser la composition.** Doctrine du dépôt : une
  police hors liste est refusée, pas remplacée. Un ISBN faux ne se voit sur aucun aperçu, il
  se voit sur cinq cents exemplaires.
- **Pas d'add-on prix.** Le second code-barres à cinq chiffres est hors périmètre. Le champ
  `prix` existe et pourra le porter plus tard ; ce n'est pas ce qui manque aujourd'hui.
- **Libre Franklin plutôt qu'OCR-B.** Aucune OCR-B parmi les 32 fichiers de `fonts/`, et la
  lecture optique tient aux barres, pas aux chiffres. Embarquer une 33ème police pour la
  seule conformité typographique du symbole n'est pas payé par ce qu'elle apporte.
- **Une image pauvre avertit, elle ne refuse pas.** Contrairement à l'ISBN : une image à
  250 ppp s'imprime, et le tirage reste juste. C'est un jugement d'auteur, pas une erreur.
- **Le silence là où l'imprimeur ne publie rien.** Le seuil de texte au dos est renseigné
  pour Lulu et KDP, qui le publient ; absent chez les quatre autres. Un seuil inventé serait
  pire que pas de contrôle — voir le relevé au § 2.3.
- **La fiche se construit sur le catalogue, jamais sur le COOKBOOK.** Celui-ci est de la
  prose relue par un humain ; en dupliquer les tableaux en données créerait deux vérités qui
  divergeraient au premier changement de guide.

## 1. Le code-barres EAN-13

### 1.1 La donnée

`Livre` gagne un champ, à côté de `prix` et `mention` :

```rust
#[serde(default)]
pub isbn: String,
```

Vide par défaut, vide autorisé — beaucoup de livres n'en ont pas, et un tirage privé n'en a
pas besoin. `VERSION` ne bouge pas : un `.ozalid` écrit avant ce chantier se relit avec un
ISBN vide, ce qui est exactement ce qu'il voulait dire.

### 1.2 La validation, et le refus

Un module neuf, `ean.rs`, sans dépendance :

- **Normaliser** : retirer tirets et espaces, majuscules pour le `X` terminal.
- **Accepter deux formes.** Treize chiffres, préfixe `978` ou `979`, clé EAN-13 vérifiée.
  Ou dix caractères, clé ISBN-10 vérifiée (modulo 11, `X` valant 10), converti en
  `978` + les neuf premiers, clé EAN-13 recalculée. L'ISBN-10 est vérifié **avant**
  conversion : sans cela un ISBN-10 faux produirait un EAN-13 valide et faux.
- **Clé EAN-13** : somme des douze premiers chiffres pondérés 1, 3, 1, 3…, puis
  `(10 - somme % 10) % 10`.
- **Refuser le reste**, en disant quoi : longueur, préfixe, ou clé. Le refus tombe à la
  composition — `Livre::verifie`, là où la police hors liste est déjà refusée.

### 1.3 Le tracé

Les 95 modules du symbole : garde `101`, six chiffres de sept modules encodés en **L** ou
**G** selon le motif que dicte le premier chiffre, garde centrale `01010`, six chiffres en
**R**, garde `101`. Le premier chiffre n'est pas barré : il ne vit que dans le motif.
Les barres de garde descendent sous la ligne des chiffres.

Zones silencieuses : onze modules à gauche, sept à droite. Elles font partie du symbole et
sont blanches — une pastille qui y déborderait rendrait le code illisible.

En Typst, une suite de `rect` posés dans la zone réglée, comme la zone ISBN est déjà un
`rect`. La largeur d'un module vaut `zone / 113` — les 95 modules du
symbole plus les 18 des zones silencieuses. La norme admet une magnification de 0,8 à 2,0 autour d'un module de
0,33 mm ; **sous 0,8, un avertissement** au compte rendu — le symbole se compose quand même,
mais il se lira mal en caisse.

Les chiffres : le premier à gauche du symbole, puis six et six sous leurs groupes, en Libre
Franklin. Au-dessus des barres, la mention `ISBN 978-2-…` avec les tirets **tels que
saisis** — c'est l'usage du livre, et les tirets d'un ISBN ne se recalculent pas sans la
table des préfixes d'éditeur.

### 1.4 L'écran

- Onglet **1 · Livre** : un champ `ISBN`, après `Prix`. Vide : « vide : pas de code-barres ».
- Onglet **2 · Couverture**, panneau « 4ème — pied et ISBN » : les cinq réglages existants ne
  bougent pas. La case « Réserver la zone ISBN » garde son nom et son sens.
- **Zone réservée et ISBN vide** → le rectangle blanc d'aujourd'hui, inchangé. Il sert :
  l'imprimeur y pose parfois le sien.
- **Zone réservée et ISBN rempli** → le code-barres, calé dans la zone.

## 2. Les contrôles avant envoi

### 2.1 La résolution effective

À la génération d'un package, pour chaque image posée — 1ère, 4ème, envoi :

- `image::dimensions()` donne les pixels — la fonction existe (`couverture.rs:620`) ;
- `image::place()` donne la géométrie en millimètres **après cadrage et zoom**, ce qui est la
  seule mesure honnête : une image recadrée à 40 % n'imprime que 40 % de ses pixels ;
- `ppp = pixels / mm × 25,4`.

Sous **300 ppp**, un avertissement au compte rendu de package, à côté de « police
introuvable » et sous la même forme. Jamais un refus.

### 2.2 Le texte au dos

La reliure gagne un champ optionnel :

```toml
[[reliure]]
# …
dos_texte_pages = 81
source = "Book Creation Guide, Cover Layout p. 17 — « If your book is 80 pages or fewer, do not include spine text. »"
```

La valeur est la **pagination minimale à laquelle le texte au dos est autorisé** — 81 chez
Lulu, qui l'interdit « à 80 pages ou moins » ; 79 chez KDP, qui l'autorise « à partir de
79 ». Les deux formulations publiées sont inverses l'une de l'autre : le champ les ramène à
une seule, et chaque `source` garde la phrase d'origine pour qu'on puisse refaire le calcul.

L'avertissement paraît quand **les deux** sont vrais : la pagination est sous le seuil, et le
dos compose au moins un élément allumé — ce que `planche::composes` sait déjà dire. Un dos nu
sous le seuil ne pose aucun problème et ne doit rien dire.

Absent du fichier : aucun contrôle, aucun message.

### 2.3 Le relevé du 28/08

| Imprimeur | Seuil | Source |
|---|---|---|
| Lulu | 81 pages | *Book Creation Guide*, Cover Layout p. 17 : « If your book is 80 pages or fewer, do not include spine text. » |
| Amazon KDP | 79 pages | Aide `G201953020` : « To include spine text, your book must have at least 79 pages. » Le seuil de 80 lu ailleurs sur la même page vaut pour leur outil *Cover Creator*, pas pour un PDF déposé. |
| BoD | non publié | recherche du 28/08, rien trouvé |
| Bookvault | non publié | leur seuil documenté est celui de la reliure, pas du dos |
| CoolLibri | non relevé | à faire, si le besoin s'en présente |
| TheBookEdition | non relevé | idem |

Deux imprimeurs sur six. C'est assumé : le champ est optionnel par construction.

## 3. La fiche de téléversement

Un `televersement.txt` écrit dans le répertoire de chaque livrable, à côté de l'intérieur, de
la planche et de la vignette, et ajouté à `Package.chemins` pour paraître au compte rendu.

Son contenu, entièrement tiré du catalogue et des mesures :

```
Titre — Auteur

Imprimeur        BoD
Format           13,5 × 21,5 cm
Reliure          Broché — dos carré collé
Papier           Crème 90 g
Finition         Pelliculage mat

Pages            164 (dont une blanche de parité)
Dos              10,91 mm
Gouttière        25,4 mm
Fond perdu       3,175 mm
Planche          238,86 × 181,35 mm

Fichiers         interieur-bod.pdf
                 couverture-bod.pdf

Le papier commandé doit être celui déclaré : c'est lui qui porte l'épaisseur du dos.
```

Suivi, s'il y en a, des avertissements du § 2 et des polices de repli — les mêmes phrases
qu'à l'écran, pour qu'un dossier relu trois mois plus tard dise ce que l'écran disait.

Une ligne n'y paraît que si elle a un contenu, comme la finition à l'écran.

## 4. Ce qui bouge ailleurs

| Fichier | Ce qui change |
|---|---|
| `projet.rs` | `Livre.isbn` ; `Livre::verifie` refuse un ISBN mal formé |
| `ean.rs` | neuf — normalisation, clé, motifs, modules |
| `couverture.rs` | la zone ISBN dessine le symbole quand l'ISBN est rempli |
| `catalogue.rs` | `Reliure.dos_texte_pages`, optionnel |
| `package.rs` | les deux contrôles, l'écriture de la fiche |
| `pods/lulu.toml`, `pods/kdp.toml` | le seuil et sa source |
| `index.html`, `app.js` | le champ ISBN |
| `README.md`, `COOKBOOK.md` | ce que l'application sait faire de plus |

## 5. Risques

- **Un code-barres illisible est pire que pas de code-barres.** La zone réglable permet de le
  réduire sous la magnification normative ; d'où l'avertissement sous 0,8. Un avertissement
  et non un refus : la zone est réglée par l'auteur, qui peut l'élargir en le lisant.
- **Le calcul de ppp dépend de `image::place`.** Si sa géométrie ne dit pas ce que je crois,
  l'avertissement se déclenchera à tort ou jamais. À vérifier sur une image de dimensions
  connues, recadrée, avant d'écrire le seuil.
- **Le seuil de 300 ppp est une convention**, pas une règle d'imprimeur relevée. Il est
  écrit une fois, nommé, et documenté comme tel.

## 6. Vérification

**Le témoin ne doit pas bouger** : aucun de ces trois chantiers ne touche la pagination.
C'est la première chose à contrôler après le chantier 2.

**Ce que les tests doivent tenir :**

- la clé EAN-13 sur des ISBN réels, et le refus d'un chiffre modifié ;
- la conversion ISBN-10 → 13, et le refus d'un ISBN-10 dont la clé ne vérifie pas ;
- les motifs L/G des dix premiers chiffres — c'est la partie que rien d'autre ne protège ;
- 95 modules, gardes aux bonnes places ;
- l'ISBN vide qui laisse le rectangle blanc, l'ISBN rempli qui pose le symbole ;
- une image dont on connaît les pixels et les millimètres, sous puis au-dessus du seuil ;
- l'avertissement de dos muet quand le dos ne compose rien, parlant quand il compose ;
- l'absence de seuil chez un POD qui n'en déclare pas ;
- la fiche : les lignes présentes, celles qui s'effacent faute de contenu.

**À l'œil**, dans l'application : un code-barres composé sur une vraie 4ème, à sa taille
réelle, et lu par un lecteur de code-barres — c'est la seule preuve qui compte.

## 7. Les lots

1. **Le code-barres** — `ean.rs`, le champ, le tracé, l'écran.
2. **Les deux contrôles** — la résolution, le seuil de dos, les deux fichiers de POD.
3. **La fiche** — reprend les avertissements du lot 2, d'où son rang.

## Hors périmètre

- L'add-on prix à cinq chiffres.
- Le code-barres sur la planche autrement que par la zone de 4ème existante.
- Une OCR-B embarquée.
- Le relevé des seuils chez CoolLibri et TheBookEdition.
- Toute conversion colorimétrique : les PDF restent ce que Typst produit.
