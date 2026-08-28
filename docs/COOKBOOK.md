# Cookbook — livrer un livre chez un imprimeur

Ozalid Studio compose l'intérieur, calcule le dos et assemble la planche. Ce qui reste
hors de l'application, c'est le compte ouvert chez l'imprimeur.

Ce document tient **un chapitre par imprimeur** : ce qu'il propose, les valeurs de son
gabarit avec la source de chacune, les réglages à saisir sur son site, et ses pièges.

Le fonctionnement de l'application est décrit dans le [README](../README.md).

## Comment lire ce document

Les fichiers `src-tauri/pods/*.toml` **font foi**. Un fichier par imprimeur, incorporé à
l'application, que votre poste peut remplacer en déposant le sien dans `<config>/pods/`.

Ce que ce cookbook ajoute : la **source** de chaque valeur, ce qu'elle vaut à l'usage, et
ce qu'on ne peut pas deviner en lisant un tableau.

Les guides et calculateurs relevés se rangent dans `build/in/editors/`, non versionné.

## Les six imprimeurs

| Imprimeur | Formats | Reliures | Papiers | Finitions | Ce qui le distingue |
|---|---|---|---|---|---|
| [Lulu](#lulu) | 16 | 2 | 3 | 2 | Le catalogue de formats le plus large |
| [BoD](#bod) | 10 | 2 | 4 | 3 | Imprimer n'oblige pas à publier |
| [Amazon KDP](#amazon-kdp) | 16 | 1 | 2 | — | La documentation technique la plus complète |
| [CoolLibri](#coollibri) | 7 | 2 | 4 | 3 | Imprimeur français, marges symétriques |
| [TheBookEdition](#thebookedition) | 9 | 2 | 3 | 3 | Production française, contrôle manuel |
| [Bookvault](#bookvault) | 3 | 1 | 3 | — | Finitions premium dès un exemplaire |

Dans la colonne « Reliures », le second chiffre compte une reliure que ces imprimeurs font
mais que l'application ne compose pas encore : elle apparaît grisée à l'écran, avec sa
raison en clair.

## Le geste, du côté de l'imprimeur

Le même chez tous les six :

1. **Déclarer le livrable** à l'étape Livraison : l'imprimeur, son format, sa reliure, son
   papier, et la finition quand il en offre une. Chez ceux qui ne publient ni formule de
   dos ni fond perdu, saisir en plus ce que vous avez relevé sur leur gabarit. Ces champs
   naissent vides ; la génération refuse en disant quoi mesurer, et à quelle pagination.
2. **Générer les packages.** Chaque livrable reçoit son répertoire : l'intérieur PDF, la
   planche PDF, une vignette PNG pour vérifier que ça tient, et un `televersement.txt` qui
   récapitule ce qu'il y a à saisir aux étapes 3 et 4 — le formulaire se remplit devant
   lui, sans revenir à la fenêtre.
3. **Créer le projet sur le site de l'imprimeur**, avec les réglages de son chapitre.
   **Le papier commandé doit être celui déclaré** : c'est lui qui porte l'épaisseur du dos.
4. **Téléverser les deux PDF**, puis contrôler l'aperçu en ligne.

## Contrôler avant de valider

Vaut chez tous les six :

- L'aperçu en ligne montre le **même nombre de pages** que la légende de la fenêtre. Un
  écart signifie que le PDF téléversé n'est pas celui qui vient d'être composé.
- Le texte du dos tombe dans le dos, sans mordre sur les plats. C'est le premier symptôme
  d'un compte de pages ou d'un papier faux.
- **Un dos mince ne porte pas de texte.** Lulu l'interdit à 80 pages ou moins, KDP le
  réserve aux livres d'au moins 79 pages ; les quatre autres ne publient rien là-dessus.
  L'application avertit au compte rendu de package chez ceux qui le publient, et se tait
  chez les autres — un seuil inventé serait pire que pas de contrôle.
- **Une image de couverture sous 300 ppp s'imprime floue.** La mesure est prise sur les
  millimètres où l'image tombe une fois cadrée et zoomée : recadrer de moitié divise la
  définition par deux. L'application la relève au compte rendu de package et se contente
  d'avertir — une image à 250 ppp s'imprime, et le tirage reste juste.
- Rien d'important ne s'approche du bord à moins de la marge de sécurité de l'imprimeur.
  C'est ce que le massicot peut emporter.
- Le titre et l'auteur sont ceux du livre.
- **La planche ne porte aucun trait de coupe ni repère de pli.** Lulu, KDP et Bookvault les
  refusent explicitement, et le fond perdu suffit à dire où couper. Le voile et le
  pointillé visibles dans l'aperçu n'entrent jamais dans le fichier remis.

Un point commun à tous les formats : le corps de texte est à 9,5 pt, interligne 1,42. Ce ne
sont pas des contraintes d'imprimeur mais des constantes de la composition, identiques
partout. La police, elle, est un réglage du livre — et elle change la pagination.

---

## Lulu

Seize formats, du poche 108 × 175 à l'A4 à l'italienne. C'est le catalogue de formats le
plus large des six.

### Ce que Lulu propose

#### Formats

| Format | Dimensions (mm) | Plafond |
|---|---|---|
| Poche 10,8 × 17,5 | 108 × 175 | — |
| Novella 12,7 × 20,3 | 127 × 203 | — |
| Digest 14 × 21,6 | 140 × 216 | — |
| A5 14,8 × 21 | 148 × 210 | — |
| US Trade 15,2 × 22,9 | 152 × 229 | — |
| Royal 15,6 × 23,4 | 156 × 234 | — |
| Comic Book 16,8 × 26 | 168 × 260 | — |
| Executive 17,8 × 25,4 | 178 × 254 | — |
| Crown Quarto 18,9 × 24,6 | 189 × 246 | — |
| Petit carré 19 × 19 | 190 × 190 | — |
| A4 21 × 29,7 | 210 × 297 | — |
| Carré 21,6 × 21,6 | 216 × 216 | — |
| US Letter 21,6 × 27,9 | 216 × 279 | — |
| Paysage 22,9 × 17,8 | 229 × 178 | 250 p. |
| US Letter à l'italienne 27,9 × 21,6 | 279 × 216 | 250 p. |
| A4 à l'italienne 29,7 × 21 | 297 × 210 | 250 p. |

**Marges identiques sur les seize** : 12,7 mm en haut, en bas et à l'extérieur.

**Gouttière**, par tranche de pagination : 12,7 mm jusqu'à 59 p. ; 15,875 jusqu'à 150 ;
25,4 jusqu'à 399 ; 28,575 jusqu'à 600 ; 31,75 au-delà.

Les trois formats à l'italienne plafonnent à 250 pages — la presse ne plie pas un paysage
plus épais. Ils ne dépassent donc jamais la troisième tranche de gouttière.

#### Reliures

| Reliure | État | Détail |
|---|---|---|
| **Broché — dos carré collé** | composée | 32 à 800 pages, nombre pair |
| Couverture rigide | grisée | L'application ne sait pas composer une couverture rigide : ni rempli, ni mors, ni cartons |

#### Papiers

| Papier | Formule de dos |
|---|---|
| Blanc non couché 60 lb | pages ÷ 17,48 + 1,524 mm |
| Crème non couché 60 lb | pages ÷ 17,48 + 1,524 mm |
| Blanc couché 80 lb | pages ÷ 17,48 + 1,524 mm |

Les trois sont publiés à 444 pages par pouce : **le dos ne dépend pas du papier** chez Lulu.

#### Finitions

Pelliculage mat, pelliculage brillant.

### Les valeurs et leurs sources

| Grandeur | Valeur | Source |
|---|---|---|
| Fond perdu | 3,175 mm (0,125 po) | guide, chapitre « Bleed Margin » |
| Dos | pages ÷ 17,48 + 1,524 mm | guide ; vérifié au générateur — 32 p → 3,35 mm, 244 p → 15,48, 800 p → 47,29 |
| Marges | 12,7 mm (0,5 po) | marge de sécurité du guide, identique sur les seize formats |
| Gouttière | cinq tranches, marge de sécurité comprise | table « Gutter Additions » |
| Pagination | 32 à 800 pages ; 32 à 250 à l'italienne | guide ; feuille de spécifications pour le plafond des paysages |

### Téléverser

| Réglage | Valeur |
|---|---|
| Format | celui du livrable — Pocketbook 4,25 × 6,875 in, US Trade 6 × 9 in, A5… |
| Reliure | Paperback (dos carré collé) |
| Encre et papier | Standard Black & White, et l'un des trois papiers |

Puis les deux fichiers : l'intérieur PDF, et la planche PDF comme couverture.

### Relever soi-même

Le générateur de gabarit répond **sans compte**, et son PDF écrit son dos, son fond perdu
et sa marge de sécurité en toutes lettres :

```
https://api.lulu.com/cover/api/v1/template/?binding_type=PB&mode=full
  &interior_width=4.25&interior_height=6.875&num_pages=244&pages_per_inch=444
  &target=print&theme=lulu2
```

La feuille de spécifications donne le reste, produit par produit :
`assets.lulu.com/media/specs/lulu-print-api-spec-sheet.xlsx`.

### Pièges

- **Deux frontières ambiguës dans la table des gouttières.** La page 60 n'appartient à
  aucune tranche (« moins de 60 », puis « 61 à 150 »), et la page 400 appartient aux deux.
  Le catalogue les donne à la tranche la **plus large** : une gouttière trop grande coûte
  des pages, une trop petite fait relier dans le texte.
- **Le Comic Book 16,8 × 26 n'existe qu'en couché 80 lb** chez Lulu. Le catalogue ne sait
  pas restreindre un papier à un format, et laissera choisir les trois.
- **Les millimètres sont ceux que Lulu publie**, pas la conversion exacte de ses pouces :
  108 × 175 là où son gabarit rend 107,95 × 174,62. L'écart va jusqu'à 0,5 mm sur le
  19 × 19, en deçà de la tolérance de rognage.
- **Le massicot mange jusqu'à 3 mm.** Une image à fond perdu s'étend bien jusqu'aux bords
  de la planche, mais ce qui compte y sera peut-être coupé.
- **Distribution commerciale** : une maquette qui imite une charte de collection existante
  reste un usage privé. Ne pas activer la Retail Distribution avec une couverture de ce
  genre.

---

## BoD

Dix formats, du 12 × 19 au 21 × 29,7 cm.

Avantage décisif pour un usage privé : **imprimer n'oblige pas à publier**, ni à prendre un
ISBN. Le parcours myBoD permet de commander pour soi sans référencer le titre.

### Ce que BoD propose

#### Formats

Les marges varient d'un format à l'autre : BoD publie un modèle Word par format.

| Format | Dimensions (mm) | Marges haut / bas / extérieur | Gouttière |
|---|---|---|---|
| 13,5 × 21,5 cm | 135 × 215 | 18,8 / 28 / 15 | 20 |
| 12 × 19 cm | 120 × 190 | 15 / 22 / 15 | 18 |
| 14,8 × 21 cm | 148 × 210 | 18,7 / 28 / 16 | 22 |
| 15,5 × 22 cm | 155 × 220 | 18,7 / 28 / 16 | 22 |
| 17 × 17 cm | 170 × 170 | 13 / 24 / 16 | 21 |
| 17 × 22 cm | 170 × 220 | 18,7 / 28 / 16 | 22 |
| 19 × 27 cm | 190 × 270 | 21 / 28 / 22 | 26 |
| 21 × 15 cm | 210 × 150 | 14 / 23 / 16 | 21 |
| 21 × 21 cm | 210 × 210 | 16 / 26 / 19 | 24 |
| 21 × 29,7 cm | 210 × 297 | 24 / 33 / 22,3 | 26 |

#### Reliures

| Reliure | État | Détail |
|---|---|---|
| **Broché — dos carré collé** | composée | 24 à 900 pages, nombre pair |
| Couverture rigide | grisée | L'application ne sait pas composer une couverture rigide : ni rempli, ni mors, ni cartons |

#### Papiers

| Papier | Formule de dos | Plafond |
|---|---|---|
| Crème 90 g *(défaut)* | pages × 0,0675 + 0,6 mm | — |
| Blanc 90 g | pages × 0,06 + 0,6 mm | — |
| Photo mat 120 g | pages × 0,063 + 0,6 mm | — |
| Photo brillant 130 g | pages × 0,0505 + 0,6 mm | **868 p.** |

Le terme constant de 0,6 mm est l'épaisseur du carton de couverture.

Le plafond du photo brillant n'est pas une question d'épaisseur : c'est au contraire le
plus mince des quatre. Il vient d'une clé du calculateur, sans raison publiée.
L'application refuse au-delà **en nommant le papier**, pour envoyer changer de papier et
non de reliure.

#### Finitions

Pelliculage mat, pelliculage brillant, pelliculage en relief.

### Les valeurs et leurs sources

| Grandeur | Valeur | Source |
|---|---|---|
| Formats, marges, gouttières | dix, format par format | modèles Word « Roman A » du 11/12/2024 |
| Fond perdu | 5 mm, commun aux dix | guide de maquette, confirmé au calculateur |
| Papiers et formules de dos | quatre, relevés un par un | calculateur officiel |
| Pagination | 24 à 900 pages, nombre pair | calculateur |

Les quatre modèles Word de BoD ne s'accordent pas partout. C'est « Roman A » qui fait foi,
et les divergences sont relevées en commentaire dans `src-tauri/pods/bod.toml`.

### Téléverser

| Réglage | Valeur |
|---|---|
| Format | celui déclaré à la Livraison — pas un autre |
| Couverture | souple, pelliculage mat, brillant ou en relief |
| Papier | celui déclaré : c'est lui qui porte l'épaisseur du dos |
| Reliure | collée |

### Pièges

- **Le dos dépend du papier, et l'écart entre les quatre est large** — du simple au tiers
  en plus, à pagination égale. Changer de papier à la commande sans refaire la couverture
  donne un dos faux.
- **Le pelliculage ne change rien à la composition** : ni géométrie, ni dos, ni marge. Les
  trois finitions sont au catalogue pour que le livrable en garde la trace. C'est une
  donnée de commande, à reporter sur le site.
- **Nombre de pages pair, sans exception** : BoD refuse un compte impair à la saisie.
  L'application ajoute au besoin une blanche de fin sans folio, et le compte affiché
  l'inclut déjà.
- **Le guide de maquette demande un export PDF/X-3:2002.** L'application rend le PDF de
  Typst, qui n'en est pas un. À surveiller au téléversement.
- **Le fond perdu de 5 mm est plus large que chez Lulu** : une composition calée sur Lulu
  perd près de 2 mm de plus au massicot.

---

## Amazon KDP

Seize formats, du 5 × 8 au 8,27 × 11,69 pouces.

La documentation technique la plus complète des six : gabarits de manuscrit officiels et
formules de dos publiées. Sa contrepartie est commerciale — **imprimer oblige à publier**,
et l'épreuve privée sort filigranée.

### Ce qu'Amazon KDP propose

#### Formats

| Format | Dimensions (mm) | Plafond propre |
|---|---|---|
| 5 × 8 po | 127 × 203,2 | — |
| 5,5 × 8,5 po | 139,7 × 215,9 | — |
| 6 × 9 po | 152,4 × 228,6 | — |
| 5,06 × 7,81 po | 128,52 × 198,37 | — |
| 5,25 × 8 po | 133,35 × 203,2 | — |
| 6,14 × 9,21 po | 155,96 × 233,93 | — |
| 6,69 × 9,61 po | 169,93 × 244,09 | — |
| 7 × 10 po | 177,8 × 254 | — |
| 7,44 × 9,69 po | 188,98 × 246,13 | — |
| 7,5 × 9,25 po | 190,5 × 234,95 | — |
| 8 × 10 po | 203,2 × 254 | — |
| 8,25 × 6 po | 209,55 × 152,4 | 750 p. |
| 8,25 × 8,25 po | 209,55 × 209,55 | 750 p. |
| 8,5 × 8,5 po | 215,9 × 215,9 | 550 p. |
| 8,5 × 11 po | 215,9 × 279,4 | 550 p. |
| 8,27 × 11,69 po | 210,06 × 296,93 | 730 p. |

**Marges identiques sur les seize** : 12,7 mm en haut, en bas et à l'extérieur.
**Gouttière** : 19,05 mm jusqu'à 700 pages, puis 22,23 mm.

La colonne des plafonds ne porte que ce que le **format** limite. Partout ailleurs, c'est
le papier qui borne — 776 pages en crème, 828 en blanc. Les deux se croisent, le plus bas
l'emporte.

Les millimètres viennent des pouces, non des centimètres arrondis que KDP affiche à côté :
le 8,27 × 11,69 fait 210,06 × 296,93 mm, et non l'A4 rond dont il s'approche.

Le 5,5 × 8,5 est à 5 mm près le 13,5 × 21,5 de BoD : une maquette faite pour l'un se
transpose presque telle quelle sur l'autre. Le 5 × 8 est le plus proche d'un poche
français. Le 8,25 × 6 est le seul format à l'italienne ; les 8,25 × 8,25 et 8,5 × 8,5 sont
les seuls carrés.

**Deux formats que KDP publie et que l'application n'offre pas.** Le 8,25 × 11 po a son
modèle Word et figure au tableau du fond perdu, mais le tableau des paginations du broché
ne le porte pas — il n'y est qu'en relié, et une pagination inventée vaudrait moins qu'une
absence. Les neuf formats de `kdp.amazon.co.jp` relèvent d'un autre marketplace, où la
couleur standard n'existe pas et où les paginations diffèrent.

#### Reliures

| Reliure | État | Détail |
|---|---|---|
| **Broché — dos carré collé** | composée | 24 à 828 pages, nombre pair |

#### Papiers

| Papier | Formule de dos | Plafond |
|---|---|---|
| Crème | pages × 0,0635 mm | **776 p.** |
| Blanc | pages × 0,0572 mm | 828 p. |

Le dos de KDP est un simple produit, **sans terme additif** : l'épaisseur de la couverture
n'entre pas dans le calcul, contrairement à Lulu et BoD. À 178 pages en 5,5 × 8,5, il fait
11,30 mm sur crème contre 10,18 sur blanc — plus d'un millimètre, assez pour faire mordre
le texte du dos sur les plats.

#### Finitions

Aucune au catalogue : le pelliculage se choisit sur le site, sans effet sur les fichiers.

### Les valeurs et leurs sources

| Grandeur | Valeur | Source |
|---|---|---|
| Fond perdu | 3,175 mm (0,125 po) | page d'aide « Create a Paperback Cover » |
| Dos, crème | pages × 0,0635 mm (0,0025 po) | idem — 280 p. → 17,78 mm |
| Dos, blanc | pages × 0,0572 mm (0,002252 po) | idem — 280 p. → 16,02 mm |
| Marges et gouttière | 12,7 mm ; 19,05 puis 22,23 mm | les dix-sept modèles Word officiels, identiques au centième |
| Pagination | 828 p. en blanc, 776 en crème, moins sur cinq formats | tableau des paginations du broché |
| Texte sur le dos | à partir de 80 pages | page d'aide couverture |

### Téléverser

| Réglage | Valeur |
|---|---|
| Format | celui déclaré à la Livraison — pas un autre |
| Reliure | Paperback, dos carré collé |
| Encre et papier | Black & white, **crème** ou **blanc** — celui sur lequel repose le calcul du dos |
| Finition | mate, l'usage pour un roman |

### Pièges

- **Le papier est définitif après publication** : il détermine l'ISBN de fabrication. Passer
  de crème à blanc impose un nouveau livre, et une couverture au dos refait.
- **En deçà de 80 pages, KDP n'imprime pas le texte du dos.** L'application l'affiche quand
  même : la planche est juste, l'imprimeur ignorera simplement ce qui s'y trouve.
- **La justification est longue sur les grands formats, et le piège grandit avec eux.** Les
  modèles gardent 12,7 mm de marge extérieure et 19,05 mm de gouttière quel que soit le
  format. En 6 × 9, la colonne fait 120,6 mm, soit environ 90 signes par ligne, contre 53
  en poche Lulu. En 8,5 × 11 elle atteint 184,2 mm, près de 140 signes. Élargir les marges
  ou grossir le corps se décide sur épreuve.
- **La marge extérieure est parmi les plus étroites des six** — 12,7 mm, contre 15 à 22,3
  chez BoD selon le format et 20 chez CoolLibri. Seul TheBookEdition descend plus bas, à
  12,5. L'aperçu en ligne signale toute composition qui déborde de la zone sûre.
- **Le blanc est légèrement sous-borné sur cinq formats.** KDP publie son plafond par couple
  format × papier ; le catalogue croise trois axes indépendants et retient le plus bas des
  deux. Il refusera donc 40 à 50 pages que KDP aurait acceptées en blanc sur ces
  formats-là. L'inverse — promettre un livre que l'imprimeur refuse — serait bien pire.

---

## CoolLibri

Sept formats, du poche 11 × 17 à l'A4 à l'italienne. Imprimeur toulousain.

### Ce que CoolLibri propose

#### Formats

| Format | Dimensions (mm) |
|---|---|
| Poche 11 × 17 | 110 × 170 |
| A5 14,8 × 21 | 148,5 × 210 |
| 16 × 24 cm | 160 × 240 |
| A5 à l'italienne 21 × 14,8 | 210 × 148 |
| Carré 21 × 21 | 210 × 210 |
| A4 21 × 29,7 | 210 × 297 |
| A4 à l'italienne 29,7 × 21 | 297 × 210 |

**Marges identiques sur les sept** : 20 mm sur les quatre côtés, gouttière comprise.

CoolLibri ne distingue pas la marge intérieure de l'extérieure et ne module pas la reliure
selon l'épaisseur. C'est le seul imprimeur du catalogue dont **tous les formats se
composent symétriquement**.

Attention à l'A5 : il fait **148,5 mm** de large, et non 148.

#### Reliures

| Reliure | État | Détail |
|---|---|---|
| **Broché — dos carré collé** | composée | 60 à 700 pages, nombre pair |
| Dos carré rembordé | grisée | L'application ne sait pas composer une couverture rigide : ni rempli, ni mors, ni cartons |

#### Papiers

| Papier | Formule de dos |
|---|---|
| Standard 90 g blanc | pages × 0,054 mm |
| Bouffant 80 g blanc | pages × 0,07143 mm |
| Crème 80 g beige | pages × 0,07143 mm |
| Couché satin 115 g blanc | pages × 0,0505 mm |

#### Finitions

Pelliculage brillant, pelliculage mat, pelliculage soft touch.

### Les valeurs et leurs sources

| Grandeur | Valeur | Source |
|---|---|---|
| Formats | sept, de 110 × 170 à 297 × 210 mm | gabarits Word officiels |
| Marges | 20 mm sur les quatre côtés | gabarits Word officiels, FAQ « 2 cm de marges tout autour » |
| Fond perdu | 3 mm | FAQ : « prévoir 3 mm de fonds perdus tournant » |
| Dos | quatre coefficients, un par papier | leur calculateur, balayé sur toute la pagination |
| Pagination | 60 à 700 pages en dos carré collé | leur configurateur |

CoolLibri publie sa formule — `(grammage / 1000) × main × (pages / 2)` — mais pas la
« main » de ses papiers. Les quatre coefficients de la table viennent donc du relevé : ils
reproduisent, une fois arrondis au millimètre, les 1 284 valeurs relevées sur les 321
paginations paires de 60 à 700 pages, sans un écart.

### Relever soi-même

L'endpoint du configurateur ne demande ni compte ni jeton. Il ne prend ni le format ni la
quantité : le dos ne dépend que de la reliure, du papier et de la pagination.

```
POST https://www.coollibri.com/Panier/ReturnSizeTranche
     ArticleId=1&OptionId=9&NumberPage=280        →  {"tranche":15}
```

`ArticleId = 1` est le dos carré collé. `OptionId` vaut 9, 10, 11 ou 12 pour les quatre
papiers, dans l'ordre du fichier `coollibri.toml`.

### Pièges

- **Leur affichage est arrondi au millimètre, le nôtre ne l'est pas.** L'incertitude qui
  reste sur le coefficient vaut moins de 0,07 mm sur un livre de 700 pages — mais c'est un
  encadrement, pas une valeur qu'ils publient.
- **Au-delà de 180 pages**, CoolLibri prévient lui-même que l'épaisseur peut varier d'une
  commande à l'autre. Le dos calculé est celui qu'ils annoncent, pas une promesse de
  fabrication : vérifier celui qu'affiche leur étape « couverture et dos » avant de valider.

---

## TheBookEdition

Neuf formats, du poche 11 × 17 à l'A4. Production française, contrôle manuel des fichiers.

TheBookEdition **ne publie aucune dimension de couverture** : ni fond perdu, ni formule de
dos. Leur gabarit est généré par leur simulateur, et leur aide en fait une condition de
recevabilité. Les valeurs en table sont donc **mesurées sur ce que ce générateur rend**.

### Ce que TheBookEdition propose

#### Formats

Les marges varient : deux sources différentes les publient, et chaque format prend celle
qui le nomme.

| Format | Dimensions (mm) | Marges haut / bas / extérieur | Gouttière |
|---|---|---|---|
| Poche 11 × 17 | 110 × 170 | 12,5 / 12,5 / 12,5 | 17,5 |
| Romantique 11 × 20 | 110 × 200 | 20 / 20 / 20 | 20 |
| Manga 12 × 18 | 120 × 180 | 12,5 / 12,5 / 12,5 | 17,5 |
| A5 14,8 × 21 | 148,5 × 210 | 12,5 / 12,5 / 12,5 | 17,5 |
| Carré 15 × 15 | 150 × 150 | 20 / 20 / 20 | 20 |
| Panoramique 19 × 15 | 190 × 150 | 20 / 20 / 20 | 20 |
| MDO 18 × 26 | 180 × 260 | 25,4 / 25,4 / 19 | 19 |
| Grand carré 21 × 21 | 210 × 210 | 20 / 20 / 20 | 20 |
| A4 21 × 29,7 | 210 × 297 | 20 / 20 / 20 | 26 |

Comme chez CoolLibri, leur A5 fait **148,5 mm** et non 148.

#### Reliures

| Reliure | État | Détail |
|---|---|---|
| **Broché — dos carré collé** | composée | 40 à 750 pages, nombre pair |
| Couverture rigide | grisée | L'application ne sait pas composer une couverture rigide : ni rempli, ni mors, ni cartons |

#### Papiers

| Papier | Formule de dos |
|---|---|
| Munken 80 g | pages × 0,06 mm |
| Papier 120 g | pages × 0,06 mm |
| Papier 135 g couleur | pages × 0,06 mm |

**Le dos ne dépend ni du papier ni du format** : les trois donnent la même épaisseur.

#### Finitions

Pelliculage brillant, pelliculage mat, pelliculage soft touch.

### Les valeurs et leurs sources

| Grandeur | Valeur | Source |
|---|---|---|
| Formats | neuf, de 110 × 170 à 210 × 297 mm | leur table des formats, qui dimensionne le gabarit |
| Fond perdu | 5 mm | mesuré : planche haute de la hauteur du livre + 10 mm, sur les neuf |
| Dos | pages × 0,060 mm | mesuré sur les gabarits JPEG 300 dpi |
| Marges | 12,5 / 17,5 au poche, au manga et à l'A5 ; 20 mm ailleurs ; 19 et 25,4 au 18 × 26 | page « Réussir la mise en page » et gabarits Word |
| Pagination | 40 à 750 pages, nombre pair | leur aide |

Le module qui génère le gabarit publie aussi en JSON la table des formats
(`action=GetFormat`) et des papiers (`action=GetWeight`), reliure par reliure. Neuf formats
× trois papiers × cinq paginations donnent le même dos à moins de 0,05 mm — l'écart
résiduel est l'arrondi au pixel.

### Relever soi-même

Un POST sans authentification sur `/fr/module/bookscover/simulationcover` renvoie un JPEG
300 dpi du gabarit complet.

### Pièges

- **Deux sources de marges qui ne s'accordent pas.** La page de conseils donne une marge de
  reliure (12,5 + 5 mm) pour le poche et l'A5, et 20 + 6 pour l'A4 ; les gabarits Word
  posent des marges symétriques sans reliure, 20 mm partout. Une marge de reliure ne
  s'invente pas là où l'imprimeur n'en publie pas.
- **Le gabarit fourni fait foi chez eux**, et un fichier qui s'en écarte est rejeté par leur
  système. Télécharger le gabarit depuis leur générateur pour la reliure, le format, le
  papier et la pagination retenus, puis vérifier la planche générée contre lui avant de
  déposer.

---

## Bookvault

Trois formats. Finitions premium dès un exemplaire.

Leur guide PDF publie le fond perdu et la gouttière, mais **pas les trois autres marges ni
la formule du dos**, calculé par leur serveur. Les épaisseurs en table viennent de leur
calculateur public.

### Ce que Bookvault propose

#### Formats

| Format | Dimensions (mm) | Marges haut / bas / extérieur | Gouttière |
|---|---|---|---|
| Novel 127 × 203 | 127 × 203 | 12,7 / 12,7 / 12,7 | 20 |
| B Format 129 × 198 | 129 × 198 | 12,7 / 12,7 / 12,7 | 20 |
| A5 148 × 210 | 148 × 210 | 20 / 20 / 20 | 20 |

Bookvault ne publiant aucune marge hors la gouttière, celles-ci sont **reprises du format
voisin déjà en table** : le KDP 5 × 8 pour les deux premiers, le CoolLibri A5 pour le
troisième. C'est un emprunt assumé, pas une mesure.

#### Reliures

| Reliure | État | Détail |
|---|---|---|
| **Broché — dos carré collé** | composée | 24 à 1000 pages, nombre pair |

#### Papiers

| Papier | Formule de dos |
|---|---|
| Crème 70 g | pages × 0,056 mm |
| Bond blanc 80 g | pages × 0,055 mm |
| Crème premium 80 g | pages × 0,072 mm |

#### Finitions

Aucune au catalogue. Bookvault en propose plusieurs, dont des finitions premium dès un
exemplaire, mais elles se choisissent sur leur site : elles ne changent ni la géométrie, ni
le dos, ni les fichiers remis.

### Les valeurs et leurs sources

| Grandeur | Valeur | Source |
|---|---|---|
| Formats | trois : Novel, B Format, A5 | calculateur |
| Fond perdu | 3 mm sur les quatre côtés de la planche | guide « Paperback Book - Cover Setup » |
| Dos | trois coefficients, un par papier | calculateur, linéaire sur sept paginations (100 p → 5,6 ; 800 → 44,8) |
| Gouttière | 20 mm — la seule marge que Bookvault impose | guide PDF, p. 2 |
| Pagination | 24 à 1000 pages | calculateur : « at least 1.3mm (24 pages) » |

Relevé sur `tools.bookvault.app/sizingcalculator`, reliure Perfect Bound, le 20/08/2026.

Leur guide cite 5,6 mm pour 100 pages de 80 g bond là où le calculateur en rend 5,5 : **le
calculateur fait foi**, c'est lui qui produit les gabarits.

### Pièges

- **Pagination en multiple de 12 moins un** (11, 23, 35, 47…) : leur système imprime un
  code-barres en dernière page, et c'est ainsi qu'on évite les blanches de fin. Cette règle
  est **incompatible** avec la parité paire que la composition impose partout ailleurs — le
  compte généré ici ne la respectera pas.
- **5 mm blancs de part et d'autre du dos** si l'intérieur de la couverture est imprimé,
  pour que la colle prenne.

---

## Ajouter un imprimeur

Un fichier, et rien d'autre. Pas de recompilation, pas de binaire à relivrer.

**Où.** `src-tauri/pods/<clé>.toml` pour un imprimeur livré avec l'application. Pour une
surcharge locale, `<config>/pods/<clé>.toml` sur votre poste : on dépose, on relance, c'est
lu. **Même clé, remplacement entier** — un fichier déposé remplace le livré, il ne s'y
fusionne pas. Devant une liste de formats, on ne saurait plus lesquels viennent d'où.

Le minimum vital : un format, une reliure composable, un papier.

```toml
cle = "exemple"
nom = "Exemple"
# Quand l'imprimeur le publie commun à ses formats. Sinon il se pose format par format,
# ou pas du tout — il se relève alors à la Livraison.
fond_perdu = 3.0

[[format]]
cle = "148x210"
nom = "A5 14,8 × 21"
mm = { largeur = 148.0, hauteur = 210.0 }
marges = { haut = 12.5, bas = 12.5, exterieur = 12.5 }
gouttieres = [ { de = 40, a = 750, mm = 17.5 } ]
source = "d'où viennent ces cinq chiffres"

[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = { min = 40, max = 750 }
parite = "paire"
source = "où la pagination admise est publiée"

[[papier]]
cle = "blanc-80"
nom = "Blanc 80 g"
teinte = "#ffffff"
dos = { forme = "multiplie", par = 0.06, plus = 0.0 }
source = "où la formule de dos est publiée"
```

Les `[[finition]]` sont facultatives et ne portent qu'une clé et un nom : une finition ne
change ni la géométrie, ni le dos, ni les marges.

La forme complète — les cinq axes, ce que chacun porte, et pourquoi ce sont des listes
plutôt qu'un arbre — est décrite au § 2 de
`docs/superpowers/specs/2026-08-26-catalogue-et-livrables-design.md`.

Un format peut porter un `pages` propre quand il plafonne plus bas que sa reliure ; un
papier aussi. Les trois bornes se croisent, et la plus basse l'emporte.

### Trois règles d'écriture

- **`source` dit d'où vient le chiffre.** Guide, gabarit, calculateur, livre réel — avec la
  date du relevé quand la valeur peut bouger. C'est la moitié de la valeur de ces fichiers,
  et c'est pourquoi ils sont en TOML : JSON n'accepte pas de commentaires.
- **`non_outille` décrit notre état, jamais celui de l'imprimeur.** « L'application ne sait
  pas composer une couverture rigide » se vérifie ; « cet imprimeur ne publie pas son
  rempli » serait une affirmation sur autrui qu'on n'a pas faite. La première phrase
  vieillit bien, la seconde devient un mensonge le jour où on regarde vraiment.
- **Une valeur d'énumération inconnue est refusée**, jamais ignorée. `geometrie` n'admet que
  `dos-carre-colle`, `parite` que `paire`, `dos.forme` que `divise`, `multiplie` ou
  `mesure`. Un champ inconnu est refusé aussi : le fichier ne doit pas pouvoir promettre
  plus que le code ne sait faire.

### Le grisé

Une reliure **sans `geometrie`** porte à la place un `non_outille`. Elle paraît alors grisée
à l'écran, sa phrase en clair sous elle. Pas une infobulle : elle se lit sans survol, et
c'est tout ce que l'utilisateur aura pour distinguer « cet imprimeur ne le fait pas » de
« l'application ne le compose pas ». Elle se rédige donc pour être lue.

Un imprimeur dont **aucune** reliure ne porte de géométrie est refusé au chargement : il ne
produirait aucun livrable, et manquerait à la liste sans que rien ne le dise.

### Ce qui refuse un fichier au chargement

De quoi écrire un fichier qui passe du premier coup. Un fichier refusé est de toute façon
nommé avec ce qui lui manque.

- **Les clés** — imprimeur, format, reliure, finition, papier — sont des noms de fichier :
  minuscules ASCII, chiffres et tirets, non vides. Elles nomment des répertoires et des
  identifiants. Deux entrées de même clé dans une même liste sont refusées.
- **Au moins un format, une reliure et un papier.** Choisir un livrable en suppose un de
  chaque, et le premier papier fait le défaut.
- **Les nombres sont finis** — TOML écrit `nan` et `inf` littéralement, et rien en aval ne
  rattrape un dos NaN. Dimensions et facteur de dos strictement positifs ; marges, fond
  perdu, gouttières et constante de dos positifs ou nuls.
- **Chaque format porte au moins une tranche de gouttière**, aucune à l'envers ni à partir
  de zéro page, et deux tranches ne se chevauchent pas.
- **Les marges tiennent dans la page** : haut + bas < hauteur, extérieur + gouttière <
  largeur sur chaque tranche. Sans ce contrôle, l'intérieur composerait une page au bloc de
  texte nul.
- **Une reliure porte une `geometrie` ou un `non_outille`**, jamais les deux, jamais aucun.
  Outillée, elle doit porter `pages` et `parite` ; non outillée, ni l'une ni l'autre.
- **Une teinte de papier vide** est refusée : le canevas la prendrait pour une couleur.
- **Un `pages` de format ou de papier doit recouvrir** la pagination de chaque reliure
  composable. Sans recouvrement, il ne composerait jamais rien avec elle, en silence.
- **Une pagination à l'envers ou à partir de zéro page** : un livre de zéro page n'existe
  pas plus qu'un format de zéro millimètre.

Un fichier de votre poste refusé **ne fait pas tomber l'application** : le catalogue
embarqué tient, et le refus s'affiche à l'étape Livraison.

### Quand on n'a pas le chiffre

- **Ne reporter que ce qui a été lu ou mesuré.** Hors tranche connue, la génération refuse
  plutôt que d'extrapoler, et le message dit quoi compléter.
- **Un imprimeur qui ne publie pas sa formule de dos entre quand même au catalogue** : on
  laisse son `fond_perdu` absent et on écrit `dos = { forme = "mesure" }`. Le dos et le fond
  perdu se saisissent alors à la Livraison, tels que relevés sur le gabarit officiel. Mieux
  vaut saisir une valeur lue qu'inscrire une formule devinée.
