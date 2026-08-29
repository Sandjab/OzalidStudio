# Reconnaissance — Livraison refondue, lot 3 (l'écran)

Date : 2026-08-30
Spec : `docs/superpowers/specs/2026-08-29-livraison-refondue-design.md`, § 5 et § 7
Lots précédents : `2026-08-29-reconnaissance-livraison-lot-1.md`,
`2026-08-29-reconnaissance-livraison-lot-2.md`

Ce que le lot 3 suppose, vérifié dans le code avant d'écrire le plan. Chaque verdict porte le
fichier et la ligne qui le fondent. Baseline relevée avant d'écrire : `cargo test`
**639 passés, 0 échec, 11 ignorés**, `node --test` **305 passés, 0 échec**. (Le handoff du
30/08 annonçait 638 : écart de décompte sur un arbre identique — `git status` est propre à
`2835104` —, pas un test qui serait apparu.)

## 1. La vue ne porte encore rien de ce que l'écran doit montrer

**1a. `LivraisonVue` ignore l'état de génération** (`commands.rs:2679-2684`). Elle porte
`livrables`, `courant`, `deja_compose` — rien d'autre. `LivrableVue`
(`commands.rs:2686-2703`) porte les quatre axes, les deux relevés et `compose:
Option<MesureVue>`. **Aucun champ ne dit si le package existe, s'il est à jour ou pourquoi il
ne l'est pas.** Tout l'écran du § 5 tient à ce champ manquant.

**1b. Le changement de signature est mécanique, et il était prévu.** `empreinte::etat(projet,
l)` (`empreinte.rs:127`) exige le `Projet` entier ; `livraison_vue(l: &Livraison)`
(`commands.rs:2718`) ne reçoit que la livraison. Son seul appel de production est
`commands.rs:2791`, dans `vue(o: &Ouvert)`, où `&o.projet` est déjà à portée : passer le
`Projet` ne coûte rien. Le commentaire d'`Etat` le dit lui-même — « `Serialize` parce que le
lot 2 le fera descendre dans la vue » (`empreinte.rs:110-111`) : le lot 2 ne l'a pas fait, la
dette revient ici, et le type est prêt.
**Verdict** : `livraison_vue(projet: &Projet)`, et `LivrableVue` reçoit un `etat: Etat`.

**1c. La forme JSON de l'état est déjà décidée, et elle est commode.** `#[serde(tag = "etat",
rename_all = "lowercase")]` (`empreinte.rs:107-108`) donne `{"etat":"jamais"}`,
`{"etat":"echec"}`, `{"etat":"ajour"}`, `{"etat":"perime","interieur":true,"couverture":false}`.
Le front lit `d.etat.etat` et, sur `perime`, les deux drapeaux — c'est exactement ce que le
§ 5 demande d'afficher (« la couverture a changé depuis cette génération »).
**Un seul manque** : `Etat::Echec` ne transporte pas son message. `Generation::Echec {
message }` le porte (`projet.rs:311-313`) mais `etat()` le laisse tomber
(`empreinte.rs:130`). Une ligne qui dit « échec » sans dire lequel oblige à regénérer pour
savoir. **Verdict** : le lot 3 doit faire remonter le message, dans `Etat::Echec` ou à côté
de lui dans `LivrableVue`.

**1d. Un test appelle `livraison_vue` directement** (`commands.rs:3157`) : il devra fabriquer
un `Projet`, pas seulement une `Livraison`.

## 2. Ce qui disparaît de l'écran, et ce que ça emporte

`livraison.js` fait 538 lignes ; le lot 3 en réécrit l'essentiel.

**2a. La ligne d'aujourd'hui est un formulaire, elle devient un compte rendu.**
`afficherLivrables` (`livraison.js:108-241`) monte par livrable trois `<select>` — reliure,
finition, papier — qui appellent tous `reglerLivrable` (`livraison.js:345`), plus un bouton
« Retirer » armé en deux temps. Ces contrôles n'ont plus de place au § 5 : la ligne n'y porte
que des faits et quatre boutons. Partent avec eux `reglerLivrable` (→ `livrable_regler`) et
l'écouteur de retrait (→ `livrable_retirer`).

**2b. La cascade à deux listes devient un formulaire à cinq.** `afficherCascade`
(`livraison.js:243`) et `afficherFormatsDuPod` (`livraison.js:257`) n'offrent qu'imprimeur ×
format ; le § 5 en veut cinq — imprimeur, format, reliure, pelliculage, papier — plus les
relevés dessous, plus **deux** verbes (`Générer`, et `Remplacer` quand on modifie). Le
balisage est à refaire : `index.html:386-388` ne déclare que deux `<select>` et un bouton, et
`app.js:1465-1466` ne branche que ceux-là.
**Ce qui se récupère tel quel** : la persistance du choix entre deux ajouts
(`livraison.js:250,266`), motivée — comparer deux papiers d'un même livre est le geste pour
lequel cet écran existe ; et le grisé de la reliure non outillée (`livraison.js:130-143`), sa
règle (« une reliure porte une géométrie **ou** une raison, jamais les deux ») et sa réserve
au README. Les deux valent pour un formulaire à cinq listes comme pour une ligne à trois.

**2c. Le compte rendu devient le corps de la ligne, et il est déjà écrit.**
`afficherPackages` (`livraison.js:390-462`) monte le `dl` à six entrées, les trois alertes
(dos rogné, police de repli, avertissements), les chemins groupés par `cheminsGroupes`
(`livraison.js:383`) et la vignette. Le § 5 déplace ce bloc dans la ligne sans en changer le
contenu. **Ce n'est pas une réécriture, c'est un déménagement** — sauf pour ce que le
verdict 3 dit ci-dessous.

**2d. `noteMesure` porte une logique que l'état rend caduque.** `perimees`
(`livraison.js:118`) se calcule aujourd'hui pour **toutes les lignes à la fois** —
`deja_compose && declares.every((x) => !x.compose)` — précisément parce que rien ne disait
ligne par ligne si la mesure valait encore. Le commentaire (`livraison.js:112-117`) explique
pourquoi le test ligne à ligne mentait alors. `etat` répond maintenant par livrable.
**Verdict** : `perimees` disparaît ; ne pas le porter tel quel dans la ligne neuve, ce serait
garder une approximation qu'on a de quoi remplacer. Attention toutefois : `etat` parle du
**package**, `compose` de la **mesure du gabarit** — les deux ne coïncident pas, un livrable
peut être mesuré sans jamais avoir été généré.

## 3. La tension centrale : le compte rendu est éphémère, la ligne doit survivre à l'ouverture

C'est le point que la spec ne traite pas, et il commande le plan.

**3a. Aujourd'hui rien de tout ça n'existe avant un clic.** `afficherPackages` est nourri par
le retour de `packager()` (`livraison.js:472-474`) : un `Vec<Resultat>`
(`commands.rs:1573-1587`) construit à la composition. La zone `#packages`
(`index.html:395`) naît `hidden`, et le reste à chaque ouverture de projet.

**3b. Ce que le `.ozalid` retient ne suffit pas à remplir la ligne du § 5.** Le modèle garde
la `Mesure` du gabarit (pages, gouttière, blanche, polices) et les deux empreintes de
`Generation::Fait` (`projet.rs:306-309`). Il ne garde **ni** `planche`, **ni** `dos_requis`,
**ni** `chemins`, **ni** `avertissements`, **ni** `interieur_partage` — tous champs de
`Package` (`package.rs:27-58`), tous produits par la composition, aucun écrit dans le projet.

**3c. Mais presque tout est reconstructible sans composer.** Le dos se recalcule déjà à chaque
vue depuis le papier et la pagination (`commands.rs:2725-2728`, et le commentaire de
`MesureVue::dos` dit pourquoi). La planche et le fond perdu sortent du catalogue et des
relevés. Les chemins se dérivent du nom : `racine.join(&cle)` pour le répertoire
(`package.rs:612`) et `package::nom(cle, quoi, ext)` pour les fichiers (`package.rs:62-64`).
La vignette elle-même est **relue du disque**, pas gardée en mémoire :
`donnee_png(Path::new(&p.vignette))` (`commands.rs:1754`, et `2580` pour les envois).
Ne se reconstruisent pas : `avertissements`, `dos_requis`, `polices_introuvables` du package,
`interieur_partage` — ce que seule la composition a vu.

**3d. Verdict, à porter au plan.** Trois voies, à trancher explicitement plutôt qu'à
découvrir en écrivant :
1. **La ligne se remplit du modèle**, et ce que seule la composition sait n'apparaît que dans
   la session qui a généré. Simple, mais une ligne change de contenu selon qu'on vient de
   générer ou de rouvrir — l'écran mentirait par omission sur un dos rogné.
2. **La ligne relit le disque à l'ouverture** (vignette, existence des fichiers). Coûte une
   lecture par livrable à chaque vue ; `envoi_vignettes` a déjà tranché ce genre d'arbitrage
   dans l'autre sens (`empreinte.rs:122-125` cite ce précédent).
3. **Le `.ozalid` retient le compte rendu** avec l'empreinte. C'est le seul moyen qu'une ligne
   rouverte dise tout ce qu'elle disait — et c'est un ajout au modèle, donc au format de
   fichier, que le lot 1 n'a pas prévu.
**Tranché avec l'utilisateur le 30/08 : (1) plus la vignette de (2).** La ligne se remplit du
modèle — pages, gouttière, dos recalculé, état — plus les chemins dérivés du nom et la
vignette relue du disque. Ce que seule la composition a vu (dos rogné, avertissements,
polices de repli, intérieur partagé) ne paraît que dans la session qui a généré. Le format de
fichier ne bouge pas.

**3e. La vignette ne descend pas dans `ProjetVue`, et le précédent le dit.** `vue()` est
rendue par **toute** commande qui écrit dans le projet ; y encoder une vignette par livrable
ferait payer une lecture de fichier et un base64 à chaque frappe qui touche le livre.
`envoi_vignettes` (`commands.rs:2409-2421`) a déjà tranché ce cas dans le bon sens : une
commande dédiée, sans cache, « demandée à l'ouverture de l'étape », avec le raisonnement écrit
au-dessus d'elle. **Verdict** : une commande `livrable_vignettes` sur ce modèle, appelée à
l'affichage de la Livraison, et non un champ de la vue.

## 4. Les trois dettes du lot 2, vérifiées au code

**4a. Le repli de papier ne laisse pas le livrable « à jour » — il le laisse *à moitié*
périmé, ce qui est pire.** La note de mémoire annonçait « il paraîtra à jour ». Vérification
faite, c'est plus retors : `normalise` remplace le papier par le premier du POD
(`projet.rs:538`) ; la clé à quatre axes change donc, et le répertoire aussi puisqu'il est
nommé par elle (`package.rs:612`). Or `empreinte::couverture` **inclut**
`l.fabrication.papier` (`empreinte.rs:80`) tandis qu'`empreinte::interieur` ne l'inclut pas —
il ne prend du gabarit que `Resolu::empreinte()`, soit format, marges et gouttières
(`catalogue.rs:1079-1096`), où le papier ne figure pas.
**Conséquence** : l'écran affichera `perime { interieur: false, couverture: true }` sur un
livrable dont **le répertoire entier** a disparu. « La couverture a changé depuis cette
génération » là où la vérité est « il n'y a plus de package du tout ».
**Aucun fichier faux n'en sort** : `interieur_du_disque` vérifie `src.is_file() &&
pdf.is_file()` (`package.rs:615`) et recomposera. C'est un mensonge d'écran, pas de PDF.
**Verdict** : le lot 3 ne peut pas se contenter d'afficher `etat`. Soit `normalise` remet le
livrable en `Generation::Jamais` quand il replie le papier (le plus honnête, et une ligne à
`projet.rs:538`), soit l'état tient compte de l'existence du répertoire. La première voie est
plus sûre : elle corrige la donnée, pas son affichage.

**4b. `nettoyage_echoue` ne peut pas atteindre l'écran aujourd'hui.** Le champ est sur
`Generation` — le type de retour, `commands.rs:1620` — et n'est rempli que par
`livrable_remplacer` (`commands.rs:1986-1994`) ; les trois autres verbes le laissent `None`
(`commands.rs:1786,1976`), et `#[serde(skip_serializing_if)]` l'efface alors de la réponse.
Il ne survit donc pas à un réaffichage : c'est un fait de la réponse, à montrer au moment du
remplacement, comme `etatPackages` montre l'attente (`livraison.js:464-486`).
**Verdict** : ligne d'état du formulaire, pas champ de la vue.

**4c. « Régénérer » qui copie au lieu de recomposer est le comportement voulu.**
`interieur_du_disque` cherche un pair du **même gabarit** dont l'empreinte d'intérieur est à
jour et dont les deux fichiers sont là (`package.rs:588-624`). Deux papiers d'un même format
partagent leur intérieur : c'est le § 4 de la spec, pas un raté. Ne pas « corriger » en
forçant la recomposition.
**Ce qui mérite d'être dit à l'écran** : `Package::interieur_partage` (`package.rs:57`)
existe déjà et n'est affiché nulle part. C'est exactement ce qui explique qu'une Régénération
ait pris trente millisecondes au lieu de dix secondes.

## 5. Le pied, le libellé, et ce que le groupement leur fait

**5a. Le pied ne bouge pas, et il ne doit pas.** `inLivrable` (`app.js:385-390`) liste les
livrables et suit `courant` ; la spec le range dans « ne bougent pas ». Mais il consomme
`libelleLivrable` (`app.js:617-626`), qui compose « gabarit — reliure — papier » où le
gabarit vaut déjà « POD — format ».

**5b. Le libellé de la ligne n'est pas celui du pied.** Sous le groupement, l'imprimeur est
porté par le groupe et « la ligne ne le répète plus » (§ 5) — le maquettage de la spec montre
« Poche 10,8 × 17,5 — Broché — Brillant — Crème non couché 60 lb », soit format, reliure,
**finition** et papier, sans le POD. `libelleLivrable` ne sait pas faire ça : il part de
`providers` dont le `libelle` porte le POD, et il n'inclut la reliure que chez un POD qui en
offre plusieurs de composables (`app.js:620-622`), règle motivée pour le pied et fausse pour
la ligne groupée. Il ne porte jamais la finition.
**Verdict** : une seconde fonction, pas une modification de `libelleLivrable` — le pied garde
sa règle, qui reste juste hors groupe. Deux libellés, deux contextes, chacun motivé.

**5c. Le groupement a besoin du nom d'imprimeur, que la vue ne donne pas directement.**
`LivrableVue.pod` est une clé (`commands.rs:2694`) ; le nom se prend dans `pods`, déjà chargé
côté front (`livraison.js:120`). L'ordre des groupes doit être « celui du premier ajout »
(§ 5) : il se dérive de l'ordre de `livrables`, qui est celui de la liste du modèle — donc
un simple regroupement stable, sans tri.

## 6. Ce que le lot coûte en tests

**6a. `packages.test.js` porte 57 tests, et une bonne moitié tient au DOM qui disparaît.**
Les tests des trois `<select>` de ligne, du bouton « Retirer » et de son armement en deux
temps (`packages.test.js:504,527,607,647,661,800,828,1606,1625`) portent sur des contrôles que
le § 5 supprime. Ceux du compte rendu (`:846` à `:1072`) portent sur un bloc qui déménage :
leur intention survit, leur sélecteur non. **À reprendre test par test, jamais en bloc** : un
test supprimé parce que son sélecteur a changé est une garantie perdue sans qu'on s'en
aperçoive.

**6b. Les trois commandes condamnées vivent ailleurs que je ne le supposais**, et c'est une
bonne nouvelle. Elles ne sont pas dans `contrats.test.js` : elles sont dans le faux backend
de `coquille.test.js` (trois `case` d'un `switch`, `:228,246,256`) et dans treize appels de
`packages.test.js`. Le faux backend devra apprendre les quatre verbes neufs avant qu'un seul
test de ligne puisse être écrit — **c'est la première tâche du lot, pas une conséquence**.

**6c. Un garde de contrat existe déjà, et c'est le filet du lot.**
`contrats.test.js:253` — « chaque commande appelée par le front est déclarée au Rust » — lit
les vrais fichiers des deux côtés. Il rougira tout seul si le front appelle un verbe que le
Rust n'expose pas, et si une commande supprimée reste appelée quelque part. Le vérifier tôt
évite de découvrir un appel oublié à l'écran.

**6d. Les tests neufs devront tenir ce que le faux DOM peut dire** : qu'un groupe porte son
imprimeur une fois, que l'ordre des groupes suit le premier ajout, que `perime.couverture`
seul n'affiche pas la même chose que `perime` sur les deux, qu'un `echec` montre son message.
Ce que le DOM ne peut pas dire est au § 9 « À l'œil » de la spec, et devra être regardé.

## 7. Ce qui restait à trancher

Les deux arbitrages de produit ont été pris avec l'utilisateur le 30/08 (verdicts 3d et
dernier point ci-dessous). Restent des décisions techniques, pour le plan :

- **Ce que devient `deja_compose`.** Il ne sert plus qu'à `perimees` (`livraison.js:118`) et à
  la veille de recomposition (`app.js:230,746`). La seconde reste ; la première meurt.
- **Le sort de `#packages`.** La spec dit « la zone intermédiaire disparaît » ; reste à savoir
  si « Tout regénérer » écrit encore quelque part, ou seulement dans les lignes.
- **Les relevés passent par Modifier → Remplacer, tranché le 30/08.** `livrable_regler`
  disparaît sans remplaçant : un relevé corrigé change le dos, donc la planche, donc le
  package d'avant est faux et recomposer est ce qu'il faut. Un seul chemin de composition,
  comme au lot 2. Le coût assumé : deux clics de plus sur un geste courant chez les POD à
  gabarit — et **une ligne du formulaire ne doit donc pas perdre les relevés déjà saisis**
  quand on ouvre Modifier, sans quoi le geste devient une ressaisie.
