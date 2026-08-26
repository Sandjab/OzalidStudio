# Le catalogue en fichiers, et le livrable à cinq axes

Date : 2026-08-26
Statut : validé (brainstorming)

## Objectif

`providers.rs` porte quatorze entrées écrites en dur, et chacune confond deux choses : un
imprimeur et un format de rognage. `kdp-55x85` n'est pas un prestataire, c'est un couple.
Cette confusion coûte trois choses. Ajouter un format oblige à recompiler et à relivrer le
binaire. Rien ne dit ce qu'un POD offre par ailleurs — reliures, finitions, papiers absents
de la table le sont sans qu'on sache s'ils n'existent pas ou si on ne les a pas outillés. Et
un livre ne peut pas être déclaré deux fois chez le même imprimeur : comparer un crème et un
blanc avant de commander oblige à basculer un réglage, regarder, revenir.

Cette spec fait du catalogue un **jeu de fichiers TOML**, un par POD, lus au lancement, et
d'un destinataire un **livrable** — une combinaison de cinq axes qu'on déclare, qu'on
compare, et qu'on retire.

Elle ne touche pas à ce que l'application compose : la planche reste celle du dos carré
collé, la seule qu'elle sache faire.

## Décisions de cadrage (brainstorming du 26/08)

- **La reliure vit dans la donnée, avec sa géométrie ou sa raison de ne pas en avoir.**
  L'application ne compose que du broché ; elle connaîtra plus de reliures qu'elle n'en
  compose, et le dira. Une reliure non outillée paraît, grisée, avec sa raison en clair. Le
  refus tombe **au moment du choix**, jamais après une couverture réglée.
- **Un livrable est une configuration**, pas une clé de prestataire. Deux livrables du même
  POD coexistent tant qu'ils diffèrent par au moins un axe.
- **Catalogue embarqué, surcharges du poste.** Les fournis par `include_str!`, les autres
  dans `<config>/pods/`. C'est le dispositif exact des maquettes (spec du 23/08), et pour
  les mêmes raisons : l'application démarre toujours, il n'y a aucun mode dégradé, et
  ajouter un POD ne demande pas de relivrer le binaire.
- **TOML, et non JSON.** C'est déjà la langue du dépôt — `livre.toml`, `preferences.toml`,
  le `maquette.toml` des archives — et JSON n'accepte pas de commentaires. Or la moitié de
  la valeur de cette table, ce sont ses commentaires de provenance.
- **Quatre listes plutôt qu'un arbre.** Le POD porte ses formats, ses reliures, ses
  finitions et ses papiers comme quatre listes indépendantes. Un arbre POD > format >
  reliure > papier obligerait à recopier les 4 papiers de BoD sous chacun de ses formats —
  40 blocs pour 4 faits, et une correction de formule à faire en dix endroits.
- **Le mécanisme d'abord, la donnée au fil de l'eau.** Un seul POD est complété à fond dans
  ce chantier — BoD — et sert de gabarit d'écriture. Le reste se remplit ensuite, un fichier
  à la fois, sans recompilation. C'est précisément ce que l'externalisation achète.

## 1. Le vocabulaire

Le mot « prestataire » disparaît. À sa place, cinq axes et un objet :

| Terme | Ce que c'est | Ce qu'il porte |
|---|---|---|
| **POD** | l'imprimeur | nom, et le fond perdu quand il est commun à ses formats |
| **Format** | un format de rognage | dimensions, marges, gouttières par tranche de pagination |
| **Reliure** | broché, rigide, spirale… | pagination admise, parité, **géométrie ou raison de ne pas en avoir** |
| **Finition** | mat, brillant, relief | rien — elle ne change pas le fichier remis |
| **Papier** | crème 90 g, blanc 90 g… | teinte à l'écran, formule de dos |
| **Livrable** | une combinaison des cinq | ce qu'on déclare à l'étape Livraison |

## 2. Le fichier d'un POD

Un fichier par POD, nommé de sa clé. Le cas courant — tout compatible avec tout — ne
s'écrit pas ; seules les exceptions se déclarent.

```toml
# pods/bod.toml
cle = "bod"
nom = "BoD (Books on Demand)"
# Publié dans le guide de maquette, commun à tous ses formats.
fond_perdu = 5.0

[[format]]
cle = "135x215"
nom = "13,5 × 21,5 cm"
mm = [135.0, 215.0]
marges = { haut = 18.8, bas = 28.0, exterieur = 15.0 }
# BoD ne module pas la reliure selon l'épaisseur : tranche unique.
gouttieres = [[24, 900, 20.0]]
source = "modèle Word « Roman » 13,5 × 21,5"

[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = [24, 900]
parite = "paire"
source = "validation du calculateur officiel"

[[reliure]]
cle = "rigide"
nom = "Couverture rigide"
non_outille = "géométrie du casewrap non relevée : rempli, mors, épaisseur des cartons"

[[finition]]
cle = "mat"
nom = "Pelliculage mat"

[[papier]]
cle = "creme-90"
nom = "Crème 90 g"
teinte = "#f7f0e0"
dos = { multiplie = 0.0675, plus = 0.6 }
source = "calculateur officiel, relevé sur 4 points — 280 p → 19,5 mm"
```

**La pagination admise vit sur la reliure, jamais sur le format** : c'est elle qui la
détermine — TheBookEdition accepte 40 à 750 pages en dos carré collé et 24 à 300 en rigide,
au même format. Les tranches de `gouttieres` du format ne sont que des tranches de marge
intérieure ; hors tranche, on refuse plutôt qu'extrapoler, comme aujourd'hui.

Trois règles d'écriture, qui prolongent celle qui tient déjà la table :

- **`source` dit d'où vient le chiffre.** Les commentaires de provenance de `providers.rs`
  descendent ici ; ils ne doivent pas rester derrière.
- **`non_outille` décrit notre état, pas celui du POD.** « géométrie non relevée » se
  vérifie ; « BoD ne publie pas son rempli » serait une affirmation sur autrui qu'on n'a pas
  faite. La nuance compte : la première phrase vieillit bien, la seconde devient un mensonge
  le jour où on regarde vraiment.
- **Une valeur d'énumération inconnue est refusée**, jamais ignorée. `geometrie` n'admet
  aujourd'hui que `dos-carre-colle`, `parite` que `paire` — les seules que le code sache
  appliquer. Un fichier qui annoncerait `parite = "multiple-12-moins-1"` serait refusé
  plutôt que d'obtenir une parité paire en silence.

Trois champs disparaissent de la table : `corps_pt`, `interligne` et `folio_pt` sont
**identiques dans les quatorze entrées actuelles**. Ce ne sont pas des faits de prestataire
mais des réglages typographiques ; ils rejoignent le livre, comme la police avant eux, avec
exactement leurs valeurs d'aujourd'hui — 9,5 pt, 1,42, 8 pt. Le témoin le prouve.

## 3. Le chargement

Au lancement : les fournis (`src-tauri/pods/*.toml`, `include_str!`), puis ceux de
`<config>/pods/`, à côté de `preferences.toml` et de `maquettes/`.

**Même clé = remplacement entier.** Le `bod.toml` du poste remplace le fourni, il ne s'y
fusionne pas. Une fusion champ par champ rendrait indéchiffrable ce que l'application lit
vraiment, et un utilisateur qui corrige une gouttière ne saurait plus quelle valeur gagne.

**Un fichier du poste fautif est refusé en le nommant** — le fichier, la ligne, ce qui
manque — et le catalogue embarqué tient. L'application démarre toujours. Le refus s'affiche
à l'étape Livraison ; un journal que personne n'ouvre laisserait l'utilisateur devant un
catalogue amputé sans savoir pourquoi.

**Un fichier embarqué fautif est un bug de compilation logique**, pas un cas d'usage : il
est attrapé par un test qui les charge tous, et le chargement peut alors échouer bruyamment.

Le catalogue est chargé une fois dans un `OnceLock`. C'est ce qui garde valides les deux
seules signatures `&'static Provider` hors tests — `commands.rs:467` et `commands.rs:1890` :
le reste du code prend déjà des références ordinaires. Le refactor est plus superficiel
qu'il n'en a l'air. `providers.rs` devient `catalogue.rs`.

## 4. Le livrable

Un livrable est identifié par les **quatre axes qui changent le fichier produit** : POD,
format, reliure, papier. La finition en est exclue — mat ou brillant donnent le même PDF.

Conséquence assumée : deux livrables qui ne différeraient que par la finition sont **refusés
à l'ajout**, parce qu'ils produiraient les mêmes octets dans deux répertoires. La finition
est portée par le livrable et paraît au récapitulatif : c'est une donnée de commande, pas de
fabrication.

Le répertoire de package suit cette identité : `bod-135x215-broche-creme90/`, et les
fichiers qu'il porte de même. Les relevés — dos et fond perdu, chez les POD qui ne les
publient pas — restent sur le livrable : ils dépendent du papier et de la pagination.

**Migration.** Les quatorze clés actuelles se convertissent sans ambiguïté : `bod` → POD `bod`,
format `135x215`, reliure `broche` ; `kdp-55x85` → `kdp`, `55x85`, `broche`. Un `.ozalid`
ancien s'ouvre, se convertit en mémoire, et repart au nouveau format au premier
enregistrement. Pas de refus, pas de perte, aucune question posée à l'utilisateur : la
conversion est totale et sans choix à faire.

## 5. La pagination appartient au gabarit d'intérieur

La pagination dépend du format (marges, gouttière), de la police, et de la parité imposée
par la reliure. Elle ne dépend **ni du papier ni de la finition** — le papier ne change que
l'épaisseur du dos.

Or le geste que cette spec veut rendre possible — comparer BoD crème et BoD blanc — crée
exactement deux livrables qui partagent tout ce qui détermine la pagination.

La `Mesure` quitte donc le destinataire pour être rangée dans le projet sous la clé
**(POD, format, reliure)**, et les livrables la lisent. Comparer deux papiers ne coûte plus une composition.
L'invariant que `projet.rs` énonce aujourd'hui — « une mesure présente vaut toujours », et ce
qui pourrait la périmer l'efface à la source — est conservé tel quel : il porte simplement
sur une clé plus large, et ce qui périme une mesure (le manuscrit, la police, le gabarit)
n'a jamais dépendu du papier.

## 6. L'écran Livraison

À l'ajout, deux listes en cascade : le POD, puis **ses** formats. Une fois la ligne posée,
trois réglages dessus : reliure, finition, papier, chacun limité à ce que ce POD offre pour
ce format.

Une reliure non outillée paraît **grisée, avec sa raison en clair sous elle**. C'est la
différence entre « ce POD ne le fait pas » et « l'application ne le compose pas », et elle
doit se lire à l'écran, pas dans un document à côté.

Le reste de l'étape ne change pas : les relevés naissent vides, le dernier livrable ne se
retire pas, et chaque package généré affiche sa planche en vignette.

## 7. Ce qui bouge ailleurs

- `package.rs` : le nom des sorties dérive du livrable et non plus d'une clé de prestataire.
  Son message de refus de pagination cite déjà « en dos carré collé » — il citera la reliure.
- `commands.rs` : `destinataire_ajouter / regler / retirer` deviennent `livrable_*`, et
  prennent une configuration au lieu d'une clé.
- `examples/temoin.rs` : `PROVIDER = "bod"` devient le triplet (`bod`, `135x215`, `broche`).
- `couverture.js`, `livraison.js`, `contrats.test.js` : la forme du destinataire change.
- `docs/COOKBOOK.md` : « Ajouter un prestataire » décrit aujourd'hui une table Rust. Il
  décrira un fichier TOML, et le chapitre BoD gagnera ses formats.

## 8. Risques

**La bascule est invisible, donc silencieuse.** Le lot 1 ne change rien à l'écran : c'est
voulu — c'est ce qui permet de prouver qu'il n'a rien cassé — mais une erreur de recopie
d'une valeur de la table ne se verrait nulle part avant un tirage. La parade est un test qui
compare, valeur par valeur, le catalogue chargé à la table actuelle, transitoire, vu passer
puis retiré ; plus le témoin.

**Le catalogue devient une donnée d'exécution.** Une faute de frappe dans un TOML ne casse
plus la compilation mais le démarrage. Même parade que pour les maquettes fournies : un test
les charge tous, et `cargo test` est exigé avant commit.

**La parité de Bookvault reste non appliquée.** Son vrai gabarit — multiple de 12 moins un —
est incompatible avec la parité paire que la composition impose partout. Le fichier écrira
`parite = "paire"`, qui est ce que l'application fait, et la réserve reste au COOKBOOK. Le
fichier ne doit pas annoncer une règle que le code n'applique pas.

**Un `.ozalid` converti ne se relit plus par l'ancienne version.** C'est le cas de toute
migration ; il est acceptable ici parce qu'aucune version n'est diffusée hors du poste.

## 9. Vérification

### Le témoin

`cargo run --example temoin` : **98 pages, dos 7,21 mm** (`PAGES_ATTENDUES`,
`examples/temoin.rs:34`). Le témoin compose en BoD — si le catalogue lu depuis un TOML ne
rend pas ce que la table rendait, ou si le corps et l'interligne déplacés vers le livre ont
changé d'un dixième, c'est là que ça se verra. À relever après chaque lot.

### Ce que les tests doivent tenir

- Tous les fichiers embarqués se chargent et se valident.
- Le catalogue chargé rend, valeur par valeur, ce que la table actuelle rendait (test
  transitoire, vu passer puis retiré au lot 1).
- Une surcharge du poste remplace le fourni de même clé, entièrement.
- Une surcharge fautive est refusée **en nommant son fichier**, et les autres se chargent.
- Une valeur d'énumération inconnue (`geometrie`, `parite`) est refusée, pas ignorée.
- Une combinaison impossible est refusée à l'ajout, avec la raison.
- Deux livrables identiques sur les quatre axes de fabrication sont refusés.
- Un `.ozalid` de l'ancien format s'ouvre converti, ses relevés intacts.
- Deux livrables du même gabarit d'intérieur ne déclenchent **qu'une** composition.
- Une reliure non outillée ne peut pas être choisie, **par le Rust**, même si l'interface
  offrait le contrôle.

Chaque test doit être vu échouer — TDD, ou mutation ciblée.

### À l'œil

Déclarer BoD crème et BoD blanc sur un livre réel, générer : deux répertoires, deux dos
différents, **une seule composition** dans la légende du pied. Puis déposer un
`<config>/pods/mon-imprimeur.toml`, relancer, et le voir dans la liste des POD sans avoir
recompilé. Enfin, y introduire une faute de frappe et vérifier que l'application démarre en
nommant le fichier fautif.

## 10. Les lots

**Lot 1 — Le catalogue en fichiers.** `catalogue.rs`, les `pods/*.toml` embarqués aux
valeurs d'aujourd'hui, le modèle à cinq axes, le chargement, les surcharges du poste, le
refus nommé. Le corps et l'interligne rejoignent le livre. Rien ne change à l'écran : la
Livraison continue d'afficher sa liste, construite depuis le nouveau catalogue.

**Lot 2 — Le livrable.** L'identité à quatre axes, le `.ozalid` migré, la mesure rangée sous
le gabarit d'intérieur, les noms de packages, les commandes `livrable_*`.

**Lot 3 — La cascade.** L'écran Livraison : POD puis format à l'ajout, reliure, finition et
papier sur la ligne, et le grisé qui dit pourquoi.

**Lot 4 — BoD complété.** Tous ses formats, papiers et reliures, chacun avec sa `source`
relevée dans ses guides — le comparatif en annonce 10 formats et 4 papiers, à vérifier chez
BoD même. Le COOKBOOK suit.

## Hors périmètre

- **La géométrie de la couverture rigide.** Rempli, mors, cartons, dos arrondi : c'est un
  chantier `planche.rs` entier, avec ses propres relevés chez chaque POD.
- **Les cinq PODs tier B et C du comparatif** — Blurb, IngramSpark, Bookelis, Pumbo,
  Booksline. On n'outille pas des imprimeurs que le comparatif dit de ne pas prendre.
- **Une interface d'édition du catalogue dans l'application.** On dépose un fichier, on
  relance. Un éditeur de gabarits serait un second produit.
