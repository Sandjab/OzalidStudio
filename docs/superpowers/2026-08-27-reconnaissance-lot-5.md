# Reconnaissance du lot 5 — Lulu, TheBookEdition et CoolLibri au complet

But du lot : faire pour Lulu, TheBookEdition et CoolLibri ce que le lot 4 a fait pour BoD —
descendre dans le catalogue tout ce que chaque imprimeur publie — **sans jamais saisir un
dos ni un fond perdu à la main**. Chaque valeur est relevée par un dispositif rejouable,
et la source est citée dans le fichier de l'imprimeur.

Relevé du 27/08/2026. **Rien de ce qui suit n'a demandé de compte**, CoolLibri compris — et
c'est le résultat le plus inattendu du lot : à l'arrivée, plus un seul des six imprimeurs
fournis ne laisse un dos ou un fond perdu à la saisie humaine.

---

## 1. Lulu — quatre sources publiques

| Source | Ce qu'on y prend |
|---|---|
| `assets.lulu.com/media/guides/en/lulu-book-creation-guide.pdf` | les formats de rognage, la marge de sécurité (0,5 po), la table des ajouts de gouttière, le fond perdu (0,125 po), la formule de dos du broché |
| `assets.lulu.com/media/specs/lulu-print-api-spec-sheet.xlsx` | 3 277 produits, révision du 21/07/2026 : dimensions en mm, paginations admises, papiers, reliures — feuille 2, colonnes `Min Page`, `Max Page`, `Trim Width (mm)`, `Bind` |
| le bundle du calculateur, `developers.lulu.com/assets/index-*.js` | l'épaisseur des papiers en pages par pouce, en clair : `{"060UW":444,"060UC":444,"070CW":460,"080CW":444,"100CW":200}` |
| `api.lulu.com/cover/api/v1/template/` | le gabarit de couverture, **sans authentification**, qui écrit son dos, son fond perdu et sa marge de sécurité en toutes lettres |

Le gabarit se rejoue ainsi, et le PDF rendu se lit à `pdftotext -layout` :

```
https://api.lulu.com/cover/api/v1/template/?binding_type=PB&mode=full
  &interior_width=4.25&interior_height=6.875&num_pages=244&pages_per_inch=444
  &target=print&theme=lulu2
```

Relevé sur les seize formats × trois paginations (32, 244, 800 pages) :

- **dos**, 444 pages/pouce : 3,35 mm / 15,48 / 47,29 — identique sur les seize formats,
  et conforme à `pages / 17,48 + 1,524`.
- **marge de sécurité** : 12,70 mm partout, sans exception.
- **fond perdu** : la planche rendue fait toujours la hauteur du livre + 2 × 3,175 mm.

Deux vérifications de contrôle : à 460 pages/pouce, 244 p → 15,00 mm ; à 200, 244 p →
32,51 mm. Les deux suivent la même formule avec l'autre diviseur — mais aucun produit
broché ne les porte, donc ils restent hors table.

### Ce que la feuille de spécifications ajoute

- **Dix-huit produits brochés**, dont deux magazines (A4 et US Letter, 460 pages/pouce)
  écartés : ils partagent le format de rognage d'un livre déjà en table et n'en diffèrent
  que par le dos, que le catalogue ne sait pas faire dépendre du produit.
- **32 à 800 pages** partout, **sauf 32 à 250 sur les trois formats à l'italienne**.
  C'est la contrainte qui a demandé le seul changement de schéma du lot.
- **Trois papiers en broché** : 60 lb non couché blanc, 60 lb non couché crème, 80 lb
  couché blanc. Tous à 444 pages par pouce, donc tous au même dos.
- Un minimum de 20 pages apparaît sur trois produits « Full Color Premium » en 80 lb
  couché ; le guide publie 32 pour tout le broché, et c'est 32 qui est retenu.

### Les deux frontières ambiguës

La table « Gutter Additions » du guide a un trou et un chevauchement :

| Publié | Ajout | Total |
|---|---|---|
| moins de 60 pages | 0 po | 0,5 po |
| 61 à 150 | 0,125 po | 0,625 po |
| 151 à 400 | 0,5 po | 1 po |
| 400 à 600 | 0,625 po | 1,125 po |
| plus de 600 | 0,75 po | 1,25 po |

La page 60 n'appartient à aucune tranche ; la page 400 appartient à deux. Les deux sont
données à la tranche **la plus large** — une gouttière trop grande coûte des pages, une
trop petite fait relier dans le texte. Un test tient ce choix (`les_deux_frontieres_ambigues_de_lulu_vont_au_plus_large`) :
le lire dans le fichier ne dirait pas s'il est encore appliqué.

### Ce qui change pour le format historique

Le `108x175` portait 14 / 15 / 13 mm de marges et une tranche unique de gouttière
(151–400 → 25 mm), dont le fichier disait lui-même que l'extérieure était « une sécurité,
non une valeur publiée ». Il prend les valeurs relevées, comme les quinze autres : 12,7 mm
partout, et les cinq tranches. Aucun témoin de non-régression n'en dépendait — celui du
dépôt est un BoD.

---

## 2. TheBookEdition — leur propre module, interrogé en JSON

`/fr/module/bookscover/simulationcover` accepte trois requêtes, toutes sans compte :

| Requête | Réponse |
|---|---|
| `ajax=true&action=GetFormat&id_binding=dosCarreColle` | les neuf formats, avec `width` et `height` en mm |
| `ajax=true&action=GetWeight&id_binding=dosCarreColle&id_color=noir` (et `couleur`) | les papiers : Munken 80 g, 120 g, et 135 g en couleur |
| `id_binding=…&id_format=…&id_color=…&id_weight=…&nb_of_page=…&submitSimulationPrice=` | le gabarit de couverture, JPEG 300 dpi |

Mesure sur 9 formats × 3 papiers × 5 paginations (40, 100, 280, 500, 750), soit 135
gabarits : le dos vaut **0,060 mm par page** et le fond perdu **5 mm**, sur toutes les
combinaisons, à moins de 0,05 mm près — l'écart résiduel est l'arrondi au pixel. Un seul
gabarit a été refusé (A5, papier couleur, 40 pages), sans message.

Les marges, elles, ont deux sources qui ne s'accordent pas :

- la page « Réussir la mise en page » nomme le poche 11 × 17 et l'A5 (12,5 mm + 5 de
  reliure), puis « un grand format comme le A4 » (20 mm + 6 de reliure) ;
- les gabarits Word de `/fr/content/24-gabarits-interieur`, un par format, posent des
  marges symétriques sans reliure — 20 mm partout, sauf 19 / 25,4 au 18 × 26 et 25 mm à
  l'A4.

Chaque format prend la source qui le nomme ; les cinq que la page de conseils ignore
prennent leur gabarit Word. Les marges des trois formats déjà en table ne bougent pas.

Les marges d'un `.doc` Word 97 se lisent sans convertisseur : le grpprl de section contient
les sprm `1F B0` (largeur), `20 B0` (hauteur), `21 B0` / `22 B0` (gauche / droite),
`23 90` / `24 90` (haut / bas), chacun suivi de sa valeur en twips sur deux octets. La
largeur retrouvée sert de contrôle : elle doit valoir le format annoncé.

---

## 3. CoolLibri — le calculateur est public, le compte inutile

Première lecture, fausse : « le dos de CoolLibri exige un compte ». Leur formule publiée —
`(grammage / 1000) × main × (pages / 2)` — l'est sans la « main » de ses papiers, et
l'épaisseur ne s'affiche qu'à l'étape « couverture et dos » du parcours de commande. Le
chantier allait s'arrêter là et demander des identifiants.

C'est l'utilisateur qui a signalé que le dos s'affiche **avant** la commande, sur
`/imprimer-un-livre`. Vérification faite, il vient d'un endpoint public — `LoadSiteTranche`
dans `/bundles/product` :

```
POST https://www.coollibri.com/Panier/ReturnSizeTranche
     ArticleId=1&OptionId=9&NumberPage=280        →  {"tranche":15}
```

`ArticleId = 1` est le dos carré collé ; `OptionId` vaut 9 à 12 pour les quatre papiers. Ni
le format ni la quantité n'entrent dans l'appel : le dos ne dépend que de la reliure, du
papier et de la pagination.

### Du millimètre entier au coefficient

La réponse est arrondie au millimètre, ce qui semble trop grossier pour en tirer une
formule. Ce n'est pas le cas : chaque **palier** encadre le coefficient, et un balayage des
321 paginations paires de 60 à 700 pages en donne entre 33 et 47 par papier. En posant
`dos = a × pages` et l'arrondi au plus proche, chaque mesure impose
`(v − 0,5) / p ≤ a < (v + 0,5) / p` :

| papier | `OptionId` | encadrement | retenu | main déduite |
|---|---|---|---|---|
| standard 90 g blanc | 9 | [0,054000 ; 0,054012] | **0,0540** | 1,200 |
| bouffant 80 g blanc | 10 | [0,071326 ; 0,071532] | **0,07143** | ≈ 1,786 |
| crème 80 g beige | 11 | le même, sur toute la plage | **0,07143** | ≈ 1,786 |
| couché satin 115 g blanc | 12 | [0,050497 ; 0,050514] | **0,0505** | ≈ 0,878 |

Les quatre coefficients reproduisent **les 1 284 mesures sans un écart**, arrondis comme
eux. Le terme constant est nul, ce qui confirme la formule publiée : la couverture n'entre
pas dans ce que CoolLibri affiche.

L'incertitude résiduelle vaut moins de 0,07 mm sur 700 pages, et c'est une incertitude
d'**affichage**, pas de reconstitution.

### Le reste, relevé de la même façon

- **Sept formats**, par leurs gabarits Word publiés (`/content/Gabarits/gabarit_word_<L>x<H>.doc`),
  lus au même décodage que ceux de TheBookEdition : 20 mm sur les quatre côtés, sans marge
  de reliure, sur les sept. L'A5 y fait **148,5 mm** de large, quand la FAQ arrondit à
  14,8 cm — et c'est le gabarit qu'on remplit. Le catalogue portait 148,0 ; la valeur est
  corrigée, la clé `148x210` ne change pas.
- **Fond perdu 3 mm**, publié dans leur FAQ : « Il faut prévoir 3 mm de fonds perdus
  tournant ».
- **Quatre papiers d'intérieur** et **trois pelliculages**, listés sur la page
  « J'imprime mon livre ».

Conséquence pour le reste du code : **plus aucun POD fourni ne porte `dos = mesure`**. La
forme reste au schéma pour les fichiers du poste, et trois tests qui prenaient CoolLibri
pour exemple d'imprimeur à gabarit ont été refondus sur des fixtures.

## 4. Le seul changement de schéma : `Format.pages`

Les trois formats à l'italienne de Lulu plafonnent à 250 pages là où leur reliure va à 800.
`Reliure.pages` ne sait pas le dire, et `Papier.pages` — ajouté au lot 4 pour le photo
brillant de BoD — porte la même idée sur un autre axe. `Format.pages` est son pendant :
même forme, même validation (bornes à l'endroit, recouvrement avec au moins une reliure
composable), même croisement.

Le croisement se fait là où le `Provider` se fabrique, et il s'y fabrique à **deux**
endroits — `aplatit` pour la table plate, `Resolu::provider` pour les commandes. Les deux
resserrent ; un test le vérifie sur les deux, parce que n'en corriger qu'un donnerait une
liste qui annonce 250 et une composition qui accepte 800.

---

## 5. Ce que le lot livre

| | avant | après |
|---|---|---|
| Lulu | 1 format, 1 papier, 0 finition | 16 formats, 3 papiers, 2 finitions, 1 reliure non outillée |
| TheBookEdition | 3 formats, 2 papiers, 0 finition | 9 formats, 3 papiers, 3 finitions, 1 reliure non outillée |
| CoolLibri | 3 formats, 1 « papier », 0 finition, ni dos ni fond perdu | 7 formats, 4 papiers, 3 finitions, dos et fond perdu calculés |
| Table plate | 23 gabarits | 48 gabarits |

Non livré, et pourquoi :

- **Les magazines de Lulu** : même format de rognage qu'un livre déjà en table, autre dos.
- **Les couvertures rigides** : `planche` ne sait pas les composer, chez aucun imprimeur.
  Elles sont déclarées non outillées chez Lulu, TheBookEdition et CoolLibri, ce qui les
  fait refuser au choix plutôt qu'après une couverture réglée.
- **Le Comic Book de Lulu en papier non couché** : le catalogue ne sait pas restreindre un
  papier à un format. La réserve est au COOKBOOK.
