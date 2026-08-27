# Reconnaissance du lot 4 — BoD complété, et le COOKBOOK

Faite le 26/08/2026 sur `catalogue-en-fichiers`, à la tête `302d916` (« Le manuscrit sait
fermer sur une biographie »), arbre propre. **Le dépôt n'a pas été touché** : lecture seule,
aucune expérience compilée.

Ce qui la distingue des deux précédentes : elle a produit un **relevé hors dépôt**. Les
lots 2 et 3 réarrangeaient de la matière déjà là ; le lot 4 en apporte, et cette matière
n'existe ni dans le code ni dans le comparatif — qui annonce « 10 formats, 4 papiers » sans
une seule valeur exploitable. Le relevé est donc la première partie du travail, et son
dispositif est décrit au § 5 pour qu'il soit rejouable le jour où BoD changera sa grille.

---

# Première partie — la carte

## 1. Ce que `bod.toml` porte aujourd'hui

Le plus pauvre des six fichiers : **un** format, **deux** reliures dont une non outillée,
**un** papier, **aucune** finition. En regard, Bookvault en porte trois, une, trois et
aucune.

```
bod.toml               formats=1 reliures=2 papiers=1 finitions=0
bookvault.toml         formats=3 reliures=1 papiers=3 finitions=0
coollibri.toml         formats=3 reliures=1 papiers=1 finitions=0
kdp.toml               formats=3 reliures=1 papiers=2 finitions=0
lulu.toml              formats=1 reliures=1 papiers=1 finitions=0
thebookedition.toml    formats=3 reliures=1 papiers=2 finitions=0
```

Ses valeurs actuelles, toutes vérifiées par le relevé de la deuxième partie : fond perdu
5 mm ; format `135x215` (135 × 215 mm, marges 18,8 / 28,0 / extérieur 15,0, gouttière
20,0 sur 24-900 pages) ; reliure `broche` en dos carré collé, 24 à 900 pages, parité paire ;
reliure `rigide` non outillée ; papier `creme-90`, dos `pages × 0,0675 + 0,6`.

## 2. Ce que le schéma admet, et ce qu'il n'admet pas

`catalogue.rs` (2 329 lignes) tient un schéma verrouillé : `deny_unknown_fields` sur chaque
structure, `Geometrie` n'admettant que `dos-carre-colle`, `Parite` que `paire`, une clé
restreinte aux minuscules, chiffres et tirets. `Pod::verifie` refuse au chargement un POD
sans format, sans reliure ou sans papier, un doublon de clé, une marge qui déborde la page,
deux tranches de gouttière qui se recouvrent, un dos non fini, et un POD dont aucune reliure
ne porte de géométrie. Une entrée mal formée ne passera pas en silence : c'est la garantie
sur laquelle ce lot s'appuie.

Le point qui compte pour la suite : **la pagination admise vit sur la reliure**
(`Reliure.pages`), jamais sur le papier ni sur le format. La spec § 2 le justifie —
TheBookEdition accepte 40-750 en dos carré collé et 24-300 en rigide, au même format. Le
verdict 1 montre que BoD contredit ce découpage.

`Pod::fabrication_defaut` prend « son premier format, sa première reliure composable, son
premier papier », et `aplatit` met cette entrée en tête. **L'ordre d'écriture du fichier est
donc un choix d'interface**, comme le lot 3 l'avait consigné.

## 3. Le COOKBOOK

316 lignes, un chapitre par prestataire. Il pointe **quatre** fois vers `providers.rs` —
l. 8 (« `src-tauri/src/providers.rs` **fait foi** »), 68 (« table `providers` »), 76
(« La compléter dans `providers.rs` »), 290 (« Une seule table à compléter : `PROVIDERS` »)
— et non trois comme le lot 3 l'avait noté. **Le fichier n'existe plus.** Le chapitre
« Ajouter un prestataire » (l. 288-305) décrit encore le monde d'avant le lot 1 : une table
Rust à compléter, la fusion de deux tables historiques, `fond_perdu: None` et `Dos::Mesure`
en syntaxe Rust. Il faut le réécrire pour le monde des fichiers déposés.

Trouvaille qui change le cadrage du lot : **le COOKBOOK porte déjà la matière du lot 4**.
Son chapitre BoD écrit « pelliculage mat, brillant ou en relief » et cite les épaisseurs des
trois papiers absents de la table — blanc 90 g à 0,012, photo mat 120 g à 0,0126, photo
brillant 130 g à 0,0101 cm par feuille. Le lot 1 les avait relevées, écrites au cookbook, et
**n'avait mis en table que le crème**. Le relevé du § 7 les confirme au chiffre près. Le
lot 4 est donc moins une découverte qu'une **descente** : ce que le cookbook dit doit
devenir ce que le catalogue porte, et le cookbook cesser de le redire.

## 4. L'écran

Le format à l'ajout est un `<select>` (`index.html:311`, rempli par `livraison.js:201`) :
dix options y tiennent sans changer une ligne. Rien à prévoir de ce côté. Le contrôle de
finition posé par le lot 3 est masqué tant que le POD n'en déclare aucune — il s'allumera
chez BoD dès que le fichier en portera, et **BoD sera le premier des six à l'allumer**.

---

# Deuxième partie — le relevé chez BoD

## 5. Le dispositif, pour qu'il soit rejouable

Trois sources, toutes publiques, aucune authentification :

1. **Les modèles Word**, `bod.fr/aide/telechargements.html` →
   `2024_Modeles_Word_pour_mise_en_page_2.zip` (990 434 octets, 41 fichiers, datés du
   11/12/2024). Quatre modèles — Roman A, Roman B, Livre pratique A, Livre pratique B —
   déclinés dans les dix formats. Les dimensions et marges se lisent dans le dernier
   `<w:sectPr>` de `word/document.xml`, en twips (1 mm = 1440/25,4 twips). Les noms de
   fichiers ne sont pas en UTF-8 : `unzip` refuse de les écrire, il faut lire l'archive en
   mémoire.
2. **La configuration du calculateur**, en clair dans le HTML de
   `bod.fr/aide/calcul-de-la-couverture.html`, attribut `data-form-fields` : la liste des
   papiers avec leur **épaisseur par feuille en cm**, celle des formats, des couvertures et
   des reliures.
3. **Le calculateur lui-même**, piloté au navigateur sur la même page. Sa réponse est du
   JSON, et elle sépare ce que l'affichage confond : `spine_width` (le papier seul),
   `thickness` (le dos total, couverture comprise) et `trimming` (le fond perdu). Lire le
   JSON plutôt que la page évite l'arrondi au dixième de millimètre qui, sur deux points,
   laissait hésiter entre 0,60 et 0,65 mm de couverture.

Les bornes de pagination, elles, sont dans `dist/js/default.js` (tables `Ql` pour les
minimums, `Zl` pour les maximums).

Une tentative d'appeler l'endpoint du calculateur en `curl` (`bod.fr/?type=1686557090`,
POST TYPO3 avec `__trustedProperties`) a échoué : 200 avec un corps vide, y compris sans
paramètres. Abandonnée après deux essais au profit du navigateur, qui est la méthode déjà
attestée par la `source` du crème 90 g.

## 6. Les dix formats et leurs marges

Relevés dans **Roman A**, le modèle dont l'entrée actuelle est tirée (verdict 3). Colonne
« intérieur » = `w:left`, qui devient la gouttière ; « extérieur » = `w:right`. Les valeurs
brutes tombent à ±0,03 mm du rond, artefact de la conversion en twips ; la colonne donne
l'arrondi au dixième.

| clé proposée | nom | mm | haut | bas | intérieur | extérieur |
|---|---|---|---|---|---|---|
| `120x190` | 12 × 19 cm | 120 × 190 | 15,0 | 22,0 | 18,0 | 15,0 |
| `135x215` | 13,5 × 21,5 cm | 135 × 215 | **18,75** | 28,0 | 20,0 | 15,0 |
| `148x210` | 14,8 × 21 cm | 148 × 210 | 18,7 | 28,0 | 22,0 | 16,0 |
| `155x220` | 15,5 × 22 cm | 155 × 220 | 18,7 | 28,0 | 22,0 | 16,0 |
| `170x170` | 17 × 17 cm | 170 × 170 | 13,0 | 24,0 | 21,0 | 16,0 |
| `170x220` | 17 × 22 cm | 170 × 220 | 18,7 | 28,0 | 22,0 | 16,0 |
| `190x270` | 19 × 27 cm | 190 × 270 | 21,0 | 28,0 | 26,0 | 22,0 |
| `210x150` | 21 × 15 cm | 210 × 150 | 14,0 | 23,0 | 21,0 | 16,0 |
| `210x210` | 21 × 21 cm | 210 × 210 | 16,0 | 26,0 | 24,0 | 19,0 |
| `210x297` | 21 × 29,7 cm | 210 × 297 | 24,0 | 33,0 | 26,0 | 22,3 |

Le fond perdu de 5 mm est confirmé par le calculateur lui-même (`trimming: 0.5` cm), sur
tous les formats interrogés : il reste commun au POD, aucune surcharge par format.

## 7. Les quatre papiers et le dos

Épaisseurs publiées dans `data-form-fields`, en cm par **feuille** ; le coefficient par
**page** en vaut la moitié, en mm. Vérifiées ensuite au calculateur sur quatre paginations
chacune (24, 100, 500, 868), en couverture souple, format 13,5 × 21,5 : la relation est
linéaire exacte, sans dérive.

| clé proposée | nom | publié (cm/feuille) | `par` (mm/page) | `plus` | relevé |
|---|---|---|---|---|---|
| `creme-90` | Crème 90 g | 0,0135 | **0,0675** | 0,6 | 24 p → 1,62 mm ; 868 p → 58,59 mm |
| `blanc-90` | Blanc 90 g | 0,012 | **0,06** | 0,6 | 24 p → 1,44 mm ; 868 p → 52,08 mm |
| `photo-mat-120` | Photo mat 120 g | 0,0126 | **0,063** | 0,6 | 24 p → 1,512 mm ; 500 p → 31,5 mm |
| `photo-brillant-130` | Photo brillant 130 g | 0,0101 | **0,0505** | 0,6 | 24 p → 1,212 mm ; 868 p → 43,834 mm |

Trois papiers de plus existent — crème 80 g et blanc 80 g « exclusivement pour éditeurs »,
photo mat blanc 90 g « exclusivement via l'interface FTP ». Hors du parcours myBoD, donc
hors table : le comparatif dit 4 papiers, et c'est le bon compte pour un auteur.

**Le terme constant est le même pour les quatre** : `thickness − spine_width = 0,06 cm`
exactement, sur toutes les mesures. C'est le carton de couverture 250 g, que le COOKBOOK
nomme déjà. Le `plus = 0.6` de la table est donc confirmé, et il n'était pas 0,65 — l'écart
que l'affichage arrondi laissait croire.

Le crème 90 g reste le défaut de BoD (`"default": "chamois"` dans la configuration du
calculateur), ce qui conforte sa place en tête de fichier.

## 8. Bornes, parité, finitions

De `default.js`, pour la couverture souple, hors clients éditeurs :

- **Minimum : 24 pages** (`Ql[Paperback].default = 24`).
- **Maximum : 900 pages**, `Zl[Paperback].default = 900` — **sauf 868 pour le photo
  brillant 130 g** (`[PhotoBrilliant]: 868`).
- **Parité paire**, sauf livret et rigide cousue qui exigent un multiple de 4.
- Couverture dure : 24 à 900 pages également, mais 52 minimum en dos rond, et **628** en
  photo brillant.

Les **trois finitions** sont confirmées par le blog de BoD : mat, brillant, en relief,
disponibles sur tous les types de couverture. Ce sont exactement les trois que le COOKBOOK
écrit déjà.

---

# Troisième partie — les verdicts

## Verdict 1 — le maximum de pages dépend du papier, et le schéma ne sait pas le dire

C'est la découverte structurante du relevé. `Reliure.pages` porte une paire min/max, et la
spec § 2 justifie qu'elle vive sur la reliure. Chez BoD, le plafond descend à **868 pages
pour le photo brillant 130 g** en couverture souple — une contrainte du papier, pas de la
reliure. Le schéma ne peut pas l'exprimer.

Quatre issues, par ordre de coût croissant :

1. **Ne pas mettre le photo brillant en table.** Trois papiers au lieu de quatre, et le
   comparatif cesse d'être tenu. Le moins coûteux, le plus mutilant.
2. **Porter 868 comme maximum de la reliure brochée.** Faux pour les trois autres papiers,
   qui perdraient 32 pages sans raison. À écarter : c'est inscrire une valeur qu'aucune
   source ne soutient.
3. **Le mettre en table avec 900, et le dire au COOKBOOK.** L'application laisserait
   composer un brillant de 880 pages que BoD refuserait à la commande. C'est un mensonge
   silencieux du même genre que ceux que le lot 3 a supprimés.
4. **Étendre le schéma : `pages` optionnel sur le papier**, qui restreint celui de la
   reliure quand il est présent. Une intersection, calculée là où la pagination se contrôle.
   C'est un changement de `catalogue.rs`, de sa validation et de `reglage_refuse`.

Mon avis : **4**, et sinon 1. C'est la seule issue qui garde la règle d'or du chantier — ne
porter que ce qui a été relevé, refuser plutôt qu'extrapoler. Mais elle élargit le lot, et
c'est un arbitrage, pas une évidence : question 1 de la quatrième partie.

## Verdict 2 — le 13,5 × 21,5 ne doit pas bouger, sinon le témoin bouge

Le modèle Roman A donne une marge haute de **18,75 mm** ; la table porte **18,8**. L'écart
est un arrondi assumé du lot 1. Le corriger en 18,75 changerait la hauteur du bloc de texte,
donc la pagination, donc le compte du témoin — et `cargo run --example temoin` est le
non-régresseur du chantier, à 98 pages et 7,21 mm de dos depuis la clôture du lot 3.

**Le format existant se recopie tel quel, à la valeur près.** Les neuf nouveaux n'ont pas ce
problème : aucun livre composé ne les utilise. Un test devrait ancrer explicitement que
`135x215` vaut toujours 18,8 — sinon un futur relevé « corrigera » le chiffre et fera bouger
le témoin sans que personne ne comprenne pourquoi.

## Verdict 3 — Roman A fait foi, et BoD s'est trompé dans trois fichiers

Les quatre modèles ne donnent pas les mêmes marges pour un même format. Trois écarts :

- **13,5 × 21,5** : Roman A donne 18,75 en haut, les trois autres 18,7. C'est Roman A qui a
  servi au lot 1 (la `source` dit « modèle Word "Roman" »), et lui seul reproduit
  l'arrondi 18,8 en table.
- **21 × 29,7** : Roman B donne une marge haute de **14,0** là où les trois autres donnent
  24,0 ; Livre pratique A donne 23,0 en extérieur là où les autres donnent 22,3.
- **21 × 15** : le fichier `Livre pratique-A-21x15cm.docx` contient une page de
  **210 × 210 mm**. Le nom annonce un format, le contenu en porte un autre. Les trois autres
  modèles donnent bien 210 × 150.

Ce ne sont pas des variantes typographiques assumées mais des fichiers mal préparés chez
BoD. **Retenir Roman A**, qui est déjà la source citée, et écrire la divergence dans la
`source` — pas dans un commentaire de fichier qui se perdra.

## Verdict 4 — les finitions : rien à relever, tout à descendre

Le lot 3 a posé le contrôle et l'a masqué faute de données, en désignant le lot 4 pour le
remplir. La matière n'est pas à chercher : mat, brillant et en relief sont au COOKBOOK
depuis le lot 1, et le blog de BoD les confirme. Trois blocs `[[finition]]` de deux lignes
chacun, et **BoD devient le seul des six à allumer le contrôle** — ce qui en fait aussi la
seule vérification à l'œil possible sur catalogue livré, là où le lot 3 devait déposer un
POD d'essai sur le poste.

`Finition` n'a que `cle` et `nom` : aucune géométrie, aucun effet sur la composition. C'est
de l'affichage, et le livrable en garde la trace. Le dire au COOKBOOK évite qu'on cherche
plus tard ce que le pelliculage change au PDF : rien.

## Verdict 5 — la raison écrite sur la reliure rigide a vieilli

`non_outille` dit aujourd'hui « géométrie du casewrap non relevée : rempli, mors, épaisseur
des cartons ». La règle d'écriture de la spec § 2 est explicite : cette phrase décrit
**notre** état, et elle doit vieillir bien.

Or le calculateur rend ces grandeurs en une requête : pour 300 pages en crème,
`wrapping: 1.7` cm, `folding: 0.8` cm, plat de 14,0 × 22,1 cm autour d'un bloc de
13,5 × 21,5. « Non relevée » devient inexact au moment où cette reconnaissance est lue.

Ce qui reste vrai, c'est que **`planche.rs` ne sait pas composer une couverture rigide**, et
la spec le range explicitement hors périmètre. La raison doit donc être réécrite pour dire
ce qui bloque vraiment — la composition, pas le relevé. Deux lignes, aucun code.

## Verdict 6 — la dette est déjà soldée, et ce verdict était faux

**Corrigé le 27/08, en cours d'exécution du lot.** Ce verdict affirmait qu'il fallait écrire
une fixture mêlant les deux formes de dos. Elle existe depuis le lot 3 :
`la_conversion_d_un_papier_suit_sa_propre_formule_de_dos` (`commands.rs:2663`), dont le
commentaire dit explicitement que `dos_publie_est_porte_par_chaque_papier` ne peut pas voir
la règle, et qui passe par `PodVue::from` — le site d'appel réel — plutôt que par
`PapierVue::from` en direct.

**La cause de l'erreur mérite d'être écrite**, parce qu'elle se reproduira : la mémoire du
lot 3 nommait ce test, et elle a été lue comme « il faudrait un tel test » au lieu de « ce
test existe et porte seul la règle ». La reconnaissance a été menée sur la mémoire plutôt
que sur la source. C'est exactement ce contre quoi la règle du dépôt met en garde — la
mémoire vieillit, le code fait foi.

Ce qui reste vrai du relevé : **BoD n'a aucun papier sans formule de dos.** Les quatre
publient une formule linéaire. La piste que le lot 3 avait consignée — « donner à BoD un
papier sans formule rendrait le test protecteur » — est donc morte, et c'est heureux : elle
aurait inscrit un mensonge dans le catalogue pour faire rougir un test.

Reste enfin une nuance sur le mot « creux ». `dos_publie_est_porte_par_chaque_papier` ancre
ce que le catalogue **livré** porte — KDP publie pour ses deux papiers, CoolLibri pour aucun
—, ce qui a sa valeur propre. Ce qu'il ne peut pas protéger, c'est la règle de portage. Les
deux tests ont leur place, et aucun n'est à écrire.

## Verdict 7 — le COOKBOOK : quatre pointeurs, un chapitre, et une redite à supprimer

Trois travaux distincts, à ne pas confondre :

1. **Les quatre pointeurs morts** (l. 8, 68, 76, 290) : `providers.rs` n'existe plus, la
   vérité est dans `src-tauri/pods/*.toml` et les surcharges du poste. Mécanique.
2. **Le chapitre « Ajouter un prestataire »** (l. 288-305) décrit une table Rust à
   compléter. Il doit décrire un fichier à écrire : la forme du TOML, les trois règles
   d'écriture de la spec § 2 (`source`, `non_outille`, le refus d'une énumération inconnue),
   le dépôt sur le poste et la relance. C'est une réécriture, pas une retouche.
3. **Le chapitre BoD redit ce que le catalogue portera.** Aujourd'hui il est la seule trace
   des trois papiers absents ; demain il ne doit plus les redire mais dire ce que le
   catalogue ne peut pas porter — le plafond à 868 pages si le verdict 1 ne l'outille pas,
   le PDF/X-3 qu'on ne produit pas, et ce que le pelliculage ne change pas.

Le lot 3 a aussi laissé deux dettes pour ce chapitre : documenter **le grisé** et **la règle
d'écriture de `non_outille`**. Elles vont au point 2.

---

# Quatrième partie — décisions

## Décidé le 27/08, les trois au plus large

1. **Le plafond du photo brillant (verdict 1) : le schéma s'étend.** Un `pages` optionnel
   sur le papier, qui restreint par intersection celui de la reliure. C'est la seule issue
   qui garde la règle du chantier — ne porter que ce qui a été relevé, refuser plutôt
   qu'extrapoler. Le quatrième papier entre en table avec son vrai plafond.
2. **Le mot « prestataire » est pris dans ce lot.** La spec § 1 le condamne depuis le
   cadrage, le COOKBOOK est rouvert de toute façon par le verdict 7, et c'est le dernier
   lot. Le lot 3 a montré ce que coûte un tel renommage — 32 commentaires Rust et 7 passages
   de README pour « destinataire » ; celui-ci porte en plus le titre du cookbook et un
   chapitre entier.
3. **Les deux dettes de fond du lot 3 sont prises.** `envois.js` bascule sur l'arbre du
   catalogue, ce qui permet de retirer `papiers` de `ProviderVue` — le motif même que le
   lot 3 a supprimé pour `dos_publie`. Et `Livraison::normalise` cesse de retirer un
   livrable sans un mot à l'ouverture, silence que la reliure réglable a le plus élargi.

Le lot 4 est donc le plus large des quatre : il ferme le chantier plutôt que de laisser des
dettes derrière lui.

## Ce que le lot livre

Dix formats, quatre papiers, trois finitions, deux reliures dont la raison du grisé est
réécrite, un plafond par papier, un test de fixture qui rend protecteur le contrôle de
`dos_publie`, un COOKBOOK qui parle du monde en fichiers, le mot « prestataire » retiré, et
les deux dettes de fond du lot 3 closes.

## Ce qu'il ne livre pas

La couverture rigide reste non composable : la spec la range hors périmètre, et le relevé du
verdict 5 ne change rien à `planche.rs`. Les cinq PODs tier B et C du comparatif restent
hors périmètre. Les trois papiers réservés aux éditeurs et à l'interface FTP restent hors
table.

## Vérification

Le témoin est le garde-fou principal : **98 pages, dos 7,21 mm**, inchangé sur tout le lot.
Le format `135x215` ne bouge pas (verdict 2) ; les neuf autres ne sont utilisés par aucun
livre composé. Une vérification à l'œil s'ajoute, la première qui n'ait pas besoin d'un POD
d'essai déposé sur le poste : ouvrir la Livraison, choisir BoD, et voir le contrôle de
finition s'allumer avec ses trois pelliculages.
