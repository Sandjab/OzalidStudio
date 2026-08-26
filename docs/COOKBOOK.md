# Cookbook — livrer un roman chez un prestataire

Ozalid Studio compose l'intérieur, calcule le dos et assemble la planche. Ce qui reste
hors de l'application, c'est le compte ouvert chez l'imprimeur : ce cookbook tient un
chapitre par prestataire — les réglages à saisir sur son site, le gabarit d'où viennent
ses constantes, et ses pièges.

`src-tauri/src/providers.rs` **fait foi** : c'est la table que l'application lit. Les
tableaux « gabarit » ci-dessous ne la redisent que pour citer la source de chaque valeur
et ce qu'elle vaut à l'usage. Les guides et calculateurs relevés se rangent dans
`build/in/editors/`, non tracké.

L'ordre est toujours le même — l'intérieur d'abord, la couverture ensuite, le dos suit la
pagination — mais il n'est plus à tenir à la main : le nombre de pages ne transite plus
par un humain. Le fonctionnement de l'application est dans `README.md`.

## Le geste, du côté du prestataire

1. Étape **Livraison** : déclarer le destinataire, son **papier**, et — pour ceux qui ne
   publient ni formule de dos ni fond perdu — ce qu'on a relevé sur leur gabarit. Les
   relevés naissent vides : la génération refuse en disant quoi mesurer, à quelle
   pagination.
2. Générer les packages. Chaque destinataire reçoit son répertoire : l'intérieur PDF, la
   planche PDF, et la vignette PNG qui sert à vérifier que ça tient.
3. Créer le projet chez le prestataire avec les réglages de son chapitre. **Le papier
   commandé doit être celui déclaré** : c'est lui qui porte l'épaisseur du dos.
4. Téléverser les deux PDF, puis contrôler l'aperçu en ligne.

## Contrôler avant de valider

Vaut chez tous les prestataires :

- L'aperçu en ligne montre le **compte de pages de la légende** de la fenêtre. Un écart
  signifie que le PDF téléversé n'est pas celui qui vient d'être composé.
- Le texte du dos tombe dans le dos, sans mordre sur les plats. C'est le premier symptôme
  d'un compte de pages ou d'un papier faux.
- Rien d'important ne s'approche du bord à moins de la marge de sécurité du prestataire —
  c'est ce que le massicot peut emporter.
- Le titre et l'auteur sont ceux du livre. La couverture est un rendu ; le `.ozalid` fait foi.
- **La planche ne porte aucun trait de coupe ni repère de pli** : Lulu, KDP et Bookvault
  les refusent explicitement, et le fond perdu suffit à dire où couper. Le voile et le
  pointillé de l'aperçu n'entrent jamais dans le fichier remis.

---

## Lulu — poche 108 × 175 mm

### Téléverser

| Réglage | Valeur |
|---|---|
| Format | Pocketbook — 4,25 × 6,875 in / 108 × 175 mm |
| Reliure | Paperback (dos carré collé) |
| Encre et papier | Standard Black & White, 60# crème non couché — le classique pour un roman |

Puis les deux fichiers : l'intérieur PDF, la planche PDF comme couverture.

### Gabarit, pour mémoire

| Grandeur | Valeur | Source |
|---|---|---|
| Format de rognage | 108 × 175 mm | guide Lulu |
| Fond perdu | 3,175 mm (0,125 po) | idem |
| Dos | pages / 17,48 + 1,524 mm | idem — vérifié sur un livre réel de 244 p. → 15,48 mm |
| Gouttière | 25 mm, **pour 151 à 400 pages seulement** | idem |
| Marge extérieure | 13 mm | sécurité |
| Marges haut / bas | 14 / 15 mm | |
| Pagination | 32 à 800 pages | table `providers` |

Corps 9,5 pt, interligne 1,42 comme partout ailleurs dans la table ; la police, elle, est
un réglage du projet et non du prestataire — et elle repagine.

### Pièges

- **Hors de la tranche 151-400 pages**, la génération refuse plutôt que d'inventer une
  gouttière. La compléter dans `providers.rs`, depuis le guide.
- **Le massicot mange jusqu'à 3 mm.** Une image à fond perdu s'étend bien jusqu'aux bords
  de la planche, mais ce qui compte y sera peut-être coupé.
- **Distribution commerciale** : une maquette qui imite une charte de collection existante
  (Folio, Blanche…) reste un usage privé. Ne pas activer la Retail Distribution avec une
  couverture de ce genre.

---

## BoD — 13,5 × 21,5 cm

Avantage décisif pour l'usage privé : **imprimer n'oblige pas à publier**, ni à prendre un
ISBN — le parcours myBoD permet de commander pour soi sans référencer le titre.

### Téléverser

| Réglage | Valeur |
|---|---|
| Format | 13,5 × 21,5 cm |
| Couverture | souple, pelliculage mat, brillant ou en relief |
| Papier | **crème 90 g** — celui sur lequel repose le calcul du dos |
| Reliure | collée |

### Gabarit, pour mémoire

| Grandeur | Valeur | Source |
|---|---|---|
| Fond perdu | 5 mm | guide de maquette BoD |
| Dos | pages × 0,0675 + 0,6 mm, en crème 90 g | calculateur officiel, relevé sur 4 points — 280 p. → 19,5 mm |
| Marge de reliure | 20 mm, quelle que soit la pagination | modèle Word « Roman » 13,5 × 21,5 |
| Marge extérieure | 15 mm | idem |
| Marges haut / bas | 18,8 / 28 mm | idem |
| Pagination | 24 à 900 pages, nombre pair | validation du calculateur |

### Pièges

- **Le dos dépend du papier.** La formule implémentée vaut pour le **crème 90 g**. Sur
  blanc 90 g (0,012 cm/feuille) le même livre donne un dos plus mince — 17,4 mm au lieu de
  19,5 à 280 pages. Changer de papier à la commande sans refaire la couverture donne un dos
  faux. Les papiers photo mat 120 g (0,0126) et brillant 130 g (0,0101) ne sont pas en table.
- **Nombre de pages pair**, sans exception : BoD refuse un compte impair à la saisie.
  L'application ajoute au besoin une blanche de fin sans folio, et le compte qu'elle
  affiche est celui à reporter, blanche comprise.
- **Le guide de maquette demande un export PDF/X-3:2002.** L'application rend le PDF de
  Typst, qui n'en est pas un ; rien dans le dépôt ne garde trace d'un dépôt réel sur ce
  point. À surveiller au téléversement.
- Le fond perdu de 5 mm est plus large que chez Lulu : une composition calée sur Lulu perd
  près de 2 mm de plus au massicot.

---

## Amazon KDP — 5 × 8, 5,5 × 8,5 ou 6 × 9 pouces

La documentation technique la plus complète du marché : gabarits de manuscrit officiels et
formules de dos publiées. Sa contrepartie est commerciale — **imprimer oblige à publier**,
et l'épreuve privée sort filigranée.

### Les trois formats outillés

| Format | Millimètres | Destinataire dans l'application |
|---|---|---|
| 5 × 8 po | 127 × 203,2 | Amazon KDP — 5 × 8 po |
| 5,5 × 8,5 po | 139,7 × 215,9 | Amazon KDP — 5,5 × 8,5 po |
| 6 × 9 po | 152,4 × 228,6 | Amazon KDP — 6 × 9 po |

Le 5,5 × 8,5 est à 5 mm près le « Roman » 135 × 215 de BoD : une maquette faite pour l'un se
transpose presque telle quelle sur l'autre. Le 5 × 8 est le plus proche d'un poche français.

### Téléverser

| Réglage | Valeur |
|---|---|
| Format | celui déclaré à la Livraison — pas un autre |
| Reliure | Paperback, dos carré collé |
| Encre et papier | Black & white, **crème** ou **blanc** — celui sur lequel repose le calcul du dos |
| Finition | mate, l'usage pour un roman |

### Gabarit, pour mémoire

| Grandeur | Valeur | Source |
|---|---|---|
| Fond perdu | 3,175 mm (0,125 po) | page d'aide « Create a Paperback Cover » |
| Dos, crème | pages × 0,0635 mm (0,0025 po) | idem — 280 p. → 17,78 mm |
| Dos, blanc | pages × 0,0572 mm (0,002252 po) | idem — 280 p. → 16,02 mm |
| Gouttière | 19,05 mm jusqu'à 700 p., puis 22,23 mm | modèles Word officiels et tableau des minimums |
| Marges haut / bas / extérieur | 12,7 mm | modèles Word officiels |
| Pagination | 24 à 828 pages | options d'impression |
| Texte sur le dos | à partir de 80 pages | page d'aide couverture |

Le dos KDP est un simple produit, **sans le terme additif** de Lulu et de BoD : l'épaisseur
de la couverture n'entre pas dans le calcul. À 178 pages en 5,5 × 8,5, il fait 11,30 mm sur
crème contre 10,18 sur blanc — plus d'un millimètre, assez pour faire mordre le texte du dos
sur les plats.

### Pièges

- **Le papier est définitif après publication** : il détermine l'ISBN de fabrication.
  Passer de crème à blanc impose un nouveau livre, et une couverture au dos refait.
- **En deçà de 80 pages, KDP n'imprime pas le texte du dos.** L'application l'affiche quand
  même : la planche est juste, l'imprimeur ignorera simplement ce qui s'y trouve.
- **La justification est longue sur les grands formats.** Les modèles gardent 12,7 mm de
  marge extérieure quel que soit le format : en 6 × 9, la colonne fait 120,6 mm, soit
  environ 90 signes par ligne au corps 9,5 pt de la table, contre 53 en poche Lulu. Élargir
  les marges ou grossir le corps se décide sur épreuve.
- **La marge extérieure est la plus étroite du lot** — 12,7 mm contre 13 chez Lulu et 15
  chez BoD. L'aperçu en ligne signale toute composition qui déborde de la zone sûre.

---

## CoolLibri — 11 × 17, A5 ou 16 × 24 cm

Imprimeur toulousain. Son intérieur est outillé ; **son dos se relève, il ne se calcule
pas** : CoolLibri publie sa formule, `(grammage / 1000) × main × (pages / 2)`, mais pas la
« main » de ses papiers. Ses gabarits de couverture publiés ne couvrent que le dos carré
rigide, en 21 × 21 et A4.

Trois formats en table, les seuls destinés au roman : **11 × 17**, **A5 14,8 × 21**,
**16 × 24 cm**.

### Relever le dos

1. Monter le projet dans le parcours en ligne de CoolLibri jusqu'à l'étape « couverture et
   dos » — c'est là qu'il s'affiche, pour le papier et la pagination retenus.
2. À la Livraison, saisir le **dos relevé** et le **fond perdu** (3 mm chez CoolLibri).

**Au-delà de 180 pages**, CoolLibri prévient lui-même que l'épaisseur peut changer :
reprendre le dos affiché à cette étape avant de générer le package.

### Gabarit, pour mémoire

| Grandeur | Valeur | Source |
|---|---|---|
| Marges | 20 mm sur les quatre côtés | gabarits Word officiels, FAQ « 2 cm de marges tout autour » |
| Fond perdu | 3 mm, à saisir | relevé sur leur gabarit |
| Dos | non calculable — à relever | formule publiée sans la « main » des papiers |
| Pagination | 60 à 700 pages en dos carré collé | selon le papier |

CoolLibri ne module pas la reliure selon l'épaisseur et ne distingue pas la marge intérieure
de l'extérieure : la composition est **symétrique**, contrairement à tous les autres
prestataires de la table.

---

## TheBookEdition — 11 × 17, 12 × 18 ou 14,8 × 21 cm

Production française, contrôle manuel des fichiers. TheBookEdition **ne publie aucune
dimension** : ni format de rognage en millimètres, ni fond perdu, ni formule de dos. Le
gabarit de couverture est généré par leur simulateur, et leur aide en fait une condition de
recevabilité.

Les valeurs en table sont donc **mesurées sur ce que leur générateur rend** (POST sur
`/fr/module/bookscover/simulationcover`, relevé le 20/08/2026), et non reconstituées :
40 p → 232,41 mm de large, 100 → 235,97, 280 → 246,80, 500 → 260,01, 750 → 275,00 au format
Poche, soit 2 × 110 + 2 × 5 de fond perdu + pages × 0,060. Les mêmes paginations sur le
papier 120 g et sur les autres formats donnent le même dos à moins de 0,04 mm — l'écart
résiduel est l'arrondi au pixel.

### Gabarit, pour mémoire

| Grandeur | Valeur | Source |
|---|---|---|
| Formats | 110 × 170, 120 × 180, **148,5** × 210 mm | leur table des formats — 148,5 et non 148, et c'est elle qui dimensionne le gabarit |
| Fond perdu | 5 mm | mesuré : planche haute de la hauteur du livre + 10 mm, sur cinq formats |
| Dos | pages × 0,060 mm, **quel que soit le papier et le format** | mesuré sur les gabarits JPEG 300 dpi |
| Papiers | Munken 80 g, 120 g | mêmes dos |
| Gouttière | 17,5 mm (12,5 de marge + 5 de reliure) | page « Réussir la mise en page » |
| Marges haut / bas / extérieur | 12,5 mm | idem |
| Pagination | 40 à 750 pages en dos carré collé, nombre pair | leur aide |

### Piège

- **Le gabarit fourni fait foi chez eux**, et un fichier qui s'en écarte est rejeté par
  leur système. Télécharger le gabarit depuis le compte auteur pour la reliure, le format,
  le papier et la pagination retenus, et vérifier la planche générée contre lui avant de
  déposer — le dos mesuré ici tient sur cinq paginations, pas sur toutes.

---

## Bookvault — 127 × 203, 129 × 198 ou A5

Finitions premium dès un exemplaire, formats libres de l'A6 au carré 297 mm. Leur guide PDF
publie le fond perdu et la gouttière, mais **pas les trois autres marges ni la formule du
dos**, calculé par leur serveur. Les épaisseurs en table viennent de leur calculateur public
(`tools.bookvault.app/sizingcalculator`, relevé le 20/08/2026, reliure Perfect Bound).

### Gabarit, pour mémoire

| Grandeur | Valeur | Source |
|---|---|---|
| Formats | Novel 127 × 203, B Format 129 × 198, A5 148 × 210 mm | calculateur |
| Fond perdu | 3 mm, sur les quatre côtés de la planche | guide « Paperback Book - Cover Setup » |
| Dos, crème 70 g | pages × 0,056 mm | calculateur, linéaire sur sept paginations (100 p → 5,6 ; 800 → 44,8) |
| Dos, bond blanc 80 g | pages × 0,055 mm | idem, trois paginations |
| Dos, crème premium 80 g | pages × 0,072 mm | idem, trois paginations |
| Gouttière | 20 mm — la seule marge que Bookvault impose | guide PDF, p. 2 |
| Marges haut / bas / extérieur | reprises du format voisin déjà en table | aucune valeur publiée |
| Pagination | 24 à 1000 pages | calculateur : « at least 1.3mm (24 pages) » |

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

## Ajouter un prestataire

Une seule table à compléter : `PROVIDERS` dans `src-tauri/src/providers.rs` — format,
marges, gouttières par tranche de pagination, réglages typographiques, fond perdu, formule
de dos et papiers. Les deux tables historiques (`index.html` pour la couverture,
`gen_interieur.py` pour l'intérieur) y ont été fusionnées : un prestataire s'ajoute
désormais à un seul endroit, et le nombre de pages ne peut plus désigner deux formats.

Deux règles tiennent l'ensemble :

- **Ne reporter que ce qui a été lu ou mesuré.** Hors tranche connue, la génération refuse
  plutôt que d'extrapoler, et le message dit quoi compléter. Ranger le guide dans
  `build/in/editors/` pour la prochaine fois.
- **Un prestataire qui ne publie pas sa formule de dos entre quand même en table**, avec
  `fond_perdu: None` et `Dos::Mesure` : le dos et le fond perdu se saisissent alors à la
  Livraison, tels que relevés sur le gabarit officiel. C'est le cas de CoolLibri. Mieux vaut
  saisir une valeur lue qu'inscrire une formule devinée.

### La file d'attente

Retenus depuis le comparatif POD du 19 août 2026, les cinq prestataires de la file sont
traités : BoD, Amazon KDP, TheBookEdition, CoolLibri et Bookvault ont chacun leur chapitre.
Les trois derniers avaient en commun de faire du gabarit qu'ils fournissent la référence
plutôt que de publier les grandeurs qui permettraient de le reconstruire ; deux ont fini par
livrer un dos mesurable sur leur propre générateur, CoolLibri non.

Lulu reste implémenté, mais le comparatif le classe en tier B : papier fin, rainage mou,
coût à l'exemplaire le plus élevé des grands POD. Son intérêt tient à l'étendue de son
catalogue de reliures.
