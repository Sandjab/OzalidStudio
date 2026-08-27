# Reconnaissance du lot 3 — la cascade

Faite le 26/08/2026 sur `catalogue-en-fichiers`, à la tête `0c84d72` (« Le dernier mot de
l'ancien monde quitte l'écran, et le plan est coché »), arbre propre. **Le dépôt n'a pas
été touché** : lecture seule, aucune expérience compilée.

Pourquoi aucune expérience, à la différence du lot 2 — le lot 2 déplaçait des types
`&'static`, changeait la forme sérialisée du `.ozalid` et migrait quatorze clés ; rien de
tout cela ne se prédit sans compiler. Le lot 3 ajoute une vue sérialisée, assouplit une
comparaison de deux champs et refait un écran. Le seul point où le compilateur avait son
mot à dire — l'assouplissement de `reglage_refuse` — se lit dans le test qui le couvre
(verdict 2). Si le plan fait mentir cette phrase, l'expérience est à monter avant, selon le
dispositif décrit en tête de la reconnaissance du lot 2.

---

# Première partie — la carte

## 1. Ce que le front connaît du catalogue

Une seule table, plate, chargée une fois au démarrage :

| | fichier:ligne |
|---|---|
| `let providers = []` | `src/app.js:20` |
| `providers = await invoke('providers_liste')` | `src/app.js:454` |

Elle est jointe **partout par la clé de gabarit** (`d.gabarit`, trois axes), jamais par la
clé du livrable :

| lecteur | fichier:ligne | ce qu'il en tire |
|---|---|---|
| `providerCourant()` | `src/app.js:491` | le gabarit visé |
| `libelleProvider(cle)` | `src/app.js:501` | le libellé d'un gabarit |
| `libelleLivrable(d)` | `src/app.js:511` | libellé + papier |
| ajout d'un livrable | `src/app.js:1291` | `pod`, `format`, `reliure`, premier papier |
| `afficherDestinataires()` | `src/livraison.js:67` | papiers, relevés, note de format |
| aperçu de couverture | `src/couverture.js:834` | le format et le fond perdu |
| canevas des envois | `src/envois.js:370` | la teinte du papier |

Conséquence pour le plan : la cascade ne peut pas se contenter d'enrichir `providers`.
Cette table est une projection POD × format ; elle n'a **pas de place** pour dire ce qu'un
POD offre d'autre.

## 2. `ProviderVue` — ce qu'elle expose, ce qu'elle tait

`src-tauri/src/commands.rs:55-101`. Elle porte : `cle` (gabarit), `pod`, `format`,
`reliure`, `libelle`, `largeur`, `hauteur`, `fond_perdu`, `dos_publie`, `papiers`
(`PapierVue { cle, libelle, teinte }`).

Elle **ne porte pas** : la liste des reliures du POD, ses finitions, ni `non_outille`.
C'est exactement ce que la cascade réclame — d'où la dette consignée à la clôture du lot 2,
« une vue neuve à tailler sur `Pod` ».

Deux détails qui comptent :

- `dos_publie` est calculé sur le **papier par défaut** du POD
  (`p.papier_defaut().dos.mm(100).is_some()`, `commands.rs:94`), pas sur le papier du
  livrable. Aujourd'hui c'est exact par accident de données : aucun POD ne mélange un
  papier à formule et un papier `Mesure` (§ 5). Le modèle, lui, le permet, et le lot 4
  élargit BoD à quatre papiers. Faire porter `dos_publie` par `PapierVue` coûte trois
  lignes pendant qu'on refait la ligne ; le laisser sur la vue plate est une dette qui
  s'ouvrira toute seule.
- `PapierVue` porte `teinte`, que seul le canevas des envois lit. Toute vue neuve qui
  prétendrait remplacer celle-ci doit la porter aussi.

## 3. L'écran Livraison, tel qu'il est

**Le balisage** — `src/index.html:292-315` : un `<h2>Destinataires</h2>`, la note de
l'étape, le bloc `#refusCatalogue`, une ligne d'ajout à **un seul** `<select>`
(`#inAjoutDestinataire`, `aria-label="Prestataire à ajouter"`) et son bouton, puis le
conteneur `#destinataires`. Le pointeur vit ailleurs, au pied de fenêtre
(`#inDestinataire`, `src/index.html:492`).

**La construction** — `afficherDestinataires()`, `src/livraison.js:67-133`. Une ligne par
livrable : le libellé du gabarit, un `<select>` de papier (`dest-papier-<clé>`, éteint à
moins de deux papiers), les relevés conditionnels (`dest-dos-*`, `dest-fp-*`, posés
seulement si `!p.dos_publie` ou `p.fond_perdu === null`), la note de format, le bouton
`dest-retirer-<clé>` (éteint s'il ne reste qu'un livrable). La liste d'ajout reçoit
**toute** la table plate, sans filtrer les gabarits déjà déclarés — c'est ce qui permet de
comparer deux papiers.

**Le renvoi au Rust** — `reglerLivrable(d)`, `src/livraison.js:156-172` : le livrable
entier voyage, `pod`/`format`/`reliure` recopiés de la ligne, le papier lu du `<select>`,
`finition: d.finition ?? null`, les deux relevés lus par un helper qui rend `null` sur un
champ vide.

**La mise en page** — `src/styles.css:1095-1141`. `.destinataire` est un `flex` qui
enveloppe ; le nom prend `min-width: 14rem`, la note `flex: 1 1 14rem` fer à droite, les
relevés passent à la ligne par `flex-basis: 100%`. Le commentaire de `src/styles.css:307`
mesure la bande à 1040 × 780 : le haut de l'étape en prend déjà 255 px, et
`.destinataire .note` a été calibrée pour que le bouton « Retirer » ne bascule pas au rang
suivant. **La ligne est pleine** : y poser reliure et finition en plus du papier la fera
déborder ou replier. C'est un vrai travail de disposition, pas un attribut à ajouter.

## 4. Les commandes et les gardes déjà en place

| commande | fichier:ligne | ce qu'elle prend |
|---|---|---|
| `providers_liste` | `commands.rs:171` | — |
| `catalogue_refus` | `commands.rs:180` | — |
| `livrable_ajouter` | `commands.rs:539` | une `Fabrication` (quatre axes) |
| `livrable_retirer` | `commands.rs:561` | une clé |
| `livrable_regler` | `commands.rs:588` | une clé + un `Livrable` |
| `livrable_viser` | `commands.rs:631` | une clé |

Les gardes, toutes antérieures au lot 3 :

- `catalogue::resout` refuse un POD, un format, une reliure ou un papier inconnu, **et
  refuse une reliure non outillée en rendant sa raison** (`catalogue.rs:786-791`).
- `reglage_refuse` (`commands.rs:521-532`) refuse un gabarit changé et une finition
  étrangère au POD.
- `livrable_ajouter` refuse le doublon sur les quatre axes (`commands.rs:546-552`).
- `livrable_retirer` refuse de retirer le dernier (`commands.rs:566-573`).
- `Livraison::normalise` (`projet.rs:411-459`) élague à l'ouverture ce que le catalogue ne
  porte plus, replie un papier disparu sur le premier du POD, et périme les mesures dont
  l'empreinte a bougé. **Elle ne tourne qu'à l'ouverture** (`projet.rs:920`).

## 5. Le catalogue réel, ce que la cascade aura à offrir

| POD | formats | reliures | dont non outillées | finitions | papiers | dos |
|---|---|---|---|---|---|---|
| BoD | 1 | 2 | 1 (`rigide`) | **0** | 1 | formule |
| Bookvault | 3 | 1 | 0 | **0** | 3 | formule |
| CoolLibri | 3 | 1 | 0 | **0** | 1 | `mesure` |
| KDP | 3 | 1 | 0 | **0** | 2 | formule |
| Lulu | 1 | 1 | 0 | **0** | 1 | formule |
| TheBookEdition | 3 | 1 | 0 | **0** | 2 | formule |

Aucune finition n'est déclarée nulle part, et une seule reliure non outillée existe. Aucun
POD ne mélange papier à formule et papier à mesure : c'est tout ou rien par POD, ce qui
sauve `dos_publie` (§ 2) sans que rien ne le garantisse.

## 6. Ce qui casse

**Rust.** L'assouplissement de `reglage_refuse` laisse
`changer_le_gabarit_d_un_livrable_est_refuse_en_disant_quoi_faire`
(`commands.rs:2354-2367`) **passer tel quel** : il exerce un **format** changé
(`6x9` → `5x8`), pas une reliure. Il faut lui ajouter le cas de la reliure réglable, vu
échouer d'abord.

**JavaScript.** Le renommage et la cascade touchent les ancrages suivants :

| fichier | `dest-*` | `inDestinataire` | `inAjoutDestinataire` / bouton | `destinataires` |
|---|---|---|---|---|
| `tests/packages.test.js` | 23 | 5 | 6 | 25 |
| `tests/coquille.test.js` | 4 | 17 | — | 8 |
| `tests/composition.test.js` | 4 | 2 | — | 2 |
| `tests/couverture.test.js` | — | 3 | — | — |

Côté source, le vocabulaire visible compte 60 occurrences : `src/app.js` 19,
`src/styles.css` 21, `src/index.html` 11, `src/livraison.js` 7, `src/couverture.js` 1,
`src/envois.js` 1. Plus le README, § « Le prestataire, choisi une seule fois »
(l. 278-300), qui définit encore le mot « destinataire ».

---

# Deuxième partie — les verdicts

## Verdict 1 — le grisé motivé est un travail d'affichage, pas de garde

L'exigence de la spec § 9 — « une reliure non outillée ne peut pas être choisie, **par le
Rust**, même si l'interface offrait le contrôle » — est **déjà tenue** :
`catalogue::resout` la refuse en rendant la raison du fichier (`catalogue.rs:786`), et le
test `resout` sur `bod`/`rigide` l'ancre (`catalogue.rs:2106`). Le lot 3 n'a donc qu'à
faire remonter `non_outille` jusqu'à l'écran et à peindre l'option grisée avec sa raison
sous elle. Le refus reste la vérité ; le grisé n'en est que la lecture.

Un test de non-régression reste utile côté front : l'option grisée ne doit pas être
choisissable, et la raison doit se lire — mais c'est un test d'écran, pas de garde.

## Verdict 2 — la reliure sur la ligne : la spec § 6 contredit le lot 2, la spec l'emporte

La spec § 6 annonce « trois réglages dessus : reliure, finition, papier ». Or la reliure
entre dans le gabarit (`cle_gabarit` = `pod-format-reliure`, `catalogue.rs:716`) et le
lot 2 a fermé la porte : « le gabarit d'un livrable ne se règle pas : retirer, puis
ajouter » (`commands.rs:521-524`). Le plan du lot 2 pose cette règle sans citer la spec
(l. 1975-1979) : c'est une garde de prudence sur ce qu'il ne traitait pas encore, pas un
arbitrage contre le § 6.

**Tranché le 26/08 : assouplir.** `reglage_refuse` ne verrouille plus que le couple
(POD, format) — les deux axes que la cascade choisit à l'ajout. Ce que le modèle fait
alors, sans qu'on ait rien à écrire :

- la clé du livrable change, comme elle change déjà pour le papier ; `livrable_regler`
  suit `courant` (`commands.rs:617-620`) et refuse le doublon sur la clé neuve ;
- la mesure de l'ancien gabarit **reste rangée sous son gabarit** — elle appartient au
  gabarit, pas au livrable, et vaut toujours pour qui le porte encore ;
- le livrable retombe sur un gabarit **sans mesure** : le pied dit « à composer », et la
  recomposition est exactement ce que la reliure exige (pagination admise, parité,
  géométrie changent avec elle) ;
- une mesure devenue orpheline survit en mémoire jusqu'à la prochaine ouverture, où
  `normalise` l'élague (`projet.rs:441-448`). Inoffensif — personne ne la lit — mais elle
  est sérialisée entre-temps. À mentionner au plan, pas à corriger : `normalise` à chaque
  mutation est un autre chantier.

Le refus « retirer, puis ajouter » **reste** pour le POD et le format ; c'est lui que le
test existant exerce, et il ne bouge pas.

## Verdict 3 — la finition : le contrôle sans la donnée

Aucun des six POD ne déclare de finition (§ 5). Le contrôle du lot 3 n'aurait donc rien à
offrir nulle part.

**Tranché le 26/08 : masqué tant que vide.** Le lot 3 pose le contrôle et ne l'affiche que
chez un POD qui déclare des finitions ; aucun relevé n'est fait dans ce lot. Le lot 4, qui
complète BoD, le remplira. Conséquence pour les tests : le cas « finition offerte » se
teste sur un POD de test — comme `pod_a_finition()` le fait déjà côté Rust
(`commands.rs:2330-2348`) — et non sur le catalogue livré.

Rien à changer côté Rust : `reglage_refuse` valide déjà la finition contre le POD
(`commands.rs:525-530`), et `Livrable.finition` est sérialisée depuis la v5
(`projet.rs:250`).

## Verdict 4 — une vue neuve **à côté** de la vue plate, pas à sa place

La cascade réclame l'arbre : POD → ses formats, ses reliures (avec `non_outille`), ses
finitions, ses papiers. La ligne réclame en plus ce que seule la projection sait dire — le
format en mm, le fond perdu effectif (`format.fond_perdu.or(pod.fond_perdu)`), le libellé
composé « POD — format ».

Remplacer `providers_liste` par une vue d'arbre obligerait `couverture.js:834`,
`envois.js:370` et trois fonctions d'`app.js` à recalculer ce que la projection leur
donne — un remaniement large, hors du geste du lot. **Ajouter** une commande d'arbre à
côté est additif : la vue plate garde ses lecteurs, la vue neuve sert la cascade et les
trois réglages de la ligne.

Ce que la vue neuve doit porter, au minimum : par POD, `cle`, `nom` ; par format, `cle`,
`nom` ; par reliure, `cle`, `nom`, `non_outille` ; par finition, `cle`, `nom` ; par papier,
`cle`, `nom`, `teinte`, et — voir § 2 — de quoi savoir si son dos se calcule.

## Verdict 5 — la ligne est pleine avant qu'on y ajoute quoi que ce soit

`.destinataire` a été calibrée au pixel pour que le bouton « Retirer » finisse sa ligne
comme celui des voisins (`styles.css:1119-1127`), et l'étape entière tient à peine un
compte rendu dans la bande (`styles.css:305-312`). Poser reliure et finition à côté du
papier n'est pas un ajout de balise : c'est une disposition à reprendre. Le plan doit
prévoir la vérification à l'œil à 1040 × 780, la fenêtre où les calibrages précédents ont
été mesurés.

## Verdict 6 — le renommage est mécanique mais large, et les tests l'ancrent

Rien de subtil : ~60 occurrences dans `src/`, ~100 ancrages dans quatre fichiers de tests,
plus une section de README à réécrire (§ 6). Le risque n'est pas la difficulté, c'est le
diff : un renommage complet noyé dans la refonte de l'écran rend la revue de la cascade
illisible. Le plan doit les séparer en tâches distinctes, commitées séparément, même s'ils
tombent dans le même lot.

Le pointeur du pied (`#inDestinataire`) est le seul id qui vive hors de l'étape Livraison
et le plus ancré des quatre (17 occurrences dans `coquille.test.js` seul) : c'est le point
où un renommage bâclé se verra le plus tard.

---

# Troisième partie — décisions et questions

## Décidé le 26/08

1. **La finition** : contrôle posé, masqué tant qu'aucune finition n'est déclarée. Aucun
   relevé chez BoD dans ce lot.
2. **La reliure** : réglable sur la ligne, conformément à la spec § 6 ;
   `reglage_refuse` ne verrouille plus que (POD, format).
3. **Le vocabulaire** : le renommage « destinataire » → « livrable » entre dans le lot 3,
   README compris, en tâche séparée.
4. **`dos_publie` par papier** et non par POD (§ 2) : corrigé dans ce lot, pendant que la
   ligne se refait. Ce qui réclame un relevé de dos suit le papier réellement choisi, et
   non le premier de la liste.
5. **Le libellé de ligne** ne gagne rien : `libelleProvider(d.gabarit)` porte déjà le POD
   et le format, et les deux axes ne se règlent plus — ils se sont choisis à l'ajout.
6. **`libelleLivrable`** (`app.js:511`), lui, porte la reliure en plus du papier. Il sert
   le pointeur du pied et les comptes rendus de package, où aucun contrôle ne se lit :
   deux livrables ne différant que par leur reliure s'y liraient identiques.

Les points 5 et 6 sont des arbitrages d'affichage pris à la reconnaissance, non des
demandes de l'utilisateur ; ils se défont sans coût si la revue les trouve faux.
