# Reconnaissance — Livraison refondue, lot 2 (les commandes)

Date : 2026-08-29
Spec : `docs/superpowers/specs/2026-08-29-livraison-refondue-design.md`
Lot précédent : `docs/superpowers/2026-08-29-reconnaissance-livraison-lot-1.md`

Ce que le lot 2 suppose, vérifié dans le code avant d'écrire le plan. Chaque verdict porte
le fichier et la ligne qui le fondent. Baseline relevée avant d'écrire : `cargo test`
**610 passés, 0 échec, 9 ignorés**.

## 1. Les quatre verbes héritent de trois commandes qui existent

**1a. `livrable_generer` = `livrable_ajouter` + une composition.** `livrable_ajouter`
(`commands.rs:668`) résout d'abord, refuse le doublon à quatre axes, pousse. Les trois gestes
restent ; ce qui s'ajoute est la composition, et l'ordre de la spec § 3 — poser puis composer —
est ce qui donne une place à l'échec.

**1b. `livrable_retirer` refuse de retirer le dernier, et la spec ne dit pas si Supprimer
en hérite.** Le refus (`commands.rs:690-700`) est motivé : « c'est lui qui donne le format sous
lequel on regarde la couverture ». Rien dans le § 3 de la spec ne l'abroge, et l'abroger
rendrait l'onglet Couverture inutilisable sans que la Livraison le dise.
**Verdict** : `livrable_supprimer` hérite du refus, et les fichiers ne sont pas effacés quand
le refus tombe — un livrable qui reste ne doit pas rester sans package.

**1c. `livrable_remplacer` n'hérite PAS de `reglage_refuse`.** Ce garde
(`commands.rs:647-660`) interdit de changer le POD et le format sur une ligne, parce que les
changer sur place laisserait le livrable sous une pagination qui n'est plus la sienne. La spec
§ 3 lève exactement cet interdit — « Rien n'interdit d'y changer l'imprimeur » — et elle le
peut parce que Remplacer **recompose**, ce que `livrable_regler` ne faisait pas. Ce qui doit
survivre du garde est son second membre : une finition que le POD ne porte pas se refuse
toujours, elle nomme une option de commande qui n'existe nulle part.

**1d. Le pointeur `courant` se rattrape à trois endroits, chacun pour sa raison.**
`livrable_retirer` le fait retomber sur le premier quand il visait le disparu
(`commands.rs:706-709`) ; `livrable_regler` le suit quand la clé change (`commands.rs:752-753`) ;
`normalise` le rebaptise quand un repli de papier renomme le livrable visé
(`projet.rs:546,569`). Les deux verbes neufs reprennent les deux premiers tels quels.

**1e. Tension à trancher dans la spec, non dans le code.** Le § 6 dit que Remplacer agit
« à sa place », le § 3 que le livrable qui change d'imprimeur part « en queue » de son nouveau
groupe. Comme les lignes d'un groupe suivront l'ordre de la liste (§ 5), les deux ne peuvent
être vrais ensemble. Le plan doit choisir : rang conservé quand le POD ne change pas, rang
poussé en fin de liste quand il change — c'est la lecture qui satisfait les deux phrases.

## 2. La composition : un seul chemin, déjà en place

**2a. `packager` (`commands.rs:1565`) est déjà « `lot` avec toutes les cibles ».** Il résout
d'abord (un axe inconnu se fige en `Resultat` d'erreur sans passer par le lot), bâtit les
`Cible` dans l'ordre des livrables, appelle `package::lot` une fois (`commands.rs:1607`), puis
recolle par `zip`. Générer un seul livrable est le même code avec une cible : rien à inventer,
et c'est ce qui garantit le « un seul jeu de garanties » du § 4.

**2b. `cible()` (`commands.rs:609`) est déjà la fonction libre partagée** par `packager` et
`envoyer`, et son commentaire dit pourquoi : « un papier avec la clé d'un autre livrable
écrirait un dos faux dans le bon répertoire ». Les verbes neufs la réutilisent, ils n'en
recopient pas les six champs.

**2c. La racine des sorties doit être vérifiée AVANT de poser le livrable.**
`sorties_racine` (`commands.rs:2254`) échoue quand le projet n'a pas encore de chemin —
« enregistrer le projet avant de composer ». Dans `packager` cet échec est global et ne
concerne aucun livrable en particulier (`commands.rs:1600-1606`). Poser le livrable puis
buter dessus laisserait dans le projet un livrable *jamais généré* qu'on n'a pas demandé,
sous un message qui parle d'autre chose. C'est le même arbitrage que `livrable_regler`, qui
résout son candidat « **avant** d'être posé ».

**2d. Rien n'est écrit sur le disque à la génération.** `vue_modifiee` (`commands.rs:2457`)
ne fait que marquer `modifie` ; le `.ozalid` n'est écrit qu'à l'enregistrement. Un livrable
généré puis abandonné sans enregistrer laisse donc ses fichiers sur le disque et son état
perdu — il rouvrira en *jamais généré*. C'est prudent dans ce sens-là (§ 4 ne réutilisera
rien), et il n'y a rien à corriger : c'est la règle de toutes les autres commandes.

## 3. L'ordre qui décide de la justesse des empreintes

**3a. `empreinte::couverture` lit la pagination retenue** — `mesure(cle_gabarit).pages`
(`empreinte.rs:64-69`) — et `packager` ne retient les mesures qu'**après** le lot
(`commands.rs:1658`). Écrire `Generation::Fait` avant `retenir_mesure` calculerait l'empreinte
sur la pagination d'avant : chaque package naîtrait périmé à la seconde même, et sur sa
couverture, c'est-à-dire sur le dos. **L'ordre est : composer, retenir la mesure, puis
seulement empreindre.** C'est le verdict le plus coûteux à ignorer du lot, et il ne se voit
dans aucune signature.

**3b. `empreinte::interieur` ne lit aucune mesure** (`empreinte.rs:33-48`) : elle est
insensible à cet ordre. Seule la couverture l'est.

**3c. `composer` retient aussi une mesure** (`commands.rs:810`), et périme donc la couverture
des livrables de ce gabarit — mais seulement si la pagination a bougé, puisque c'est le
nombre de pages qui entre dans l'empreinte, non le fait d'avoir composé. C'est le
comportement voulu : une pagination qui bouge fait un dos qui bouge.

## 4. La réutilisation de l'intérieur depuis le disque

**4a. `lot` (`package.rs:581`) a déjà tout ce qu'il faut** : il reçoit `projet` et `racine`.
Les livrables générés, leurs empreintes et leurs répertoires se lisent depuis ces deux
arguments. Aucune signature à élargir, aucun appelant à retoucher.

**4b. `Mesure` porte exactement ce qui manque à `InterieurCompose`.** Celui-ci
(`package.rs:69-77`) réclame `pages`, `gouttiere`, `blanche`, `polices_introuvables`, `src`,
`pdf` ; `Mesure` (`projet.rs:386-403`) porte les quatre premiers, et les deux chemins se
reconstruisent par `nom()` dans le répertoire du livrable candidat. L'amorce est donc un
montage, pas une lecture de PDF.

**4c. Une mesure absente ne peut pas être un faux négatif.** `oublier_mesures` n'a que trois
appelants — `modifier_livre`, `modifier_interieur`, `remplacer_texte` (`projet.rs:860,866,873`)
— et ces trois-là changent tous `empreinte::interieur`, qui contient le livre, les réglages
d'intérieur et le texte. Une mesure effacée va donc toujours avec une empreinte qui a bougé :
l'amorce se refusera d'elle-même, jamais à tort.

**4d. Les cibles de la passe doivent être exclues du vivier d'amorce.** Sans cette exclusion,
`livrable_regenerer` sur un livrable à jour se retrouverait lui-même comme candidat, copierait
son PDF sur lui-même — le garde-fou `de != vers` (`package.rs:421`) évite la troncature, mais
ne recompose rien — et rendrait un package annoncé `interieur_partage` sans qu'aucune
composition ait eu lieu. Régénérer ne régénérerait jamais. L'exclusion a un second effet, qui
la recommande seule : « Tout regénérer » a toutes les cibles dans la passe, donc aucune amorce,
donc **exactement le comportement d'aujourd'hui**.

**4e. Deux fichiers, pas un.** `assembler` copie le `.typ` et le `.pdf` (`package.rs:414-426`).
La condition d'amorce doit exiger les deux présents : un `.typ` manquant laisserait dans le
répertoire livré une source qui ne correspond à rien.

**4f. Aucun test existant de `lot` n'est à réécrire.** Les deux qui l'appellent
(`package.rs:1677` et `1755`) partent de `Projet::nouveau`, dont les livrables portent
`Generation::Jamais` (`projet.rs:355-365`) : aucun candidat, aucune amorce, même résultat.
Le moule du test neuf est le leur — `#[ignore = "lance le sidecar Typst : cargo test --
--ignored"]` (`package.rs:1622,1676`), Typst du PATH avec `avec_polices(fonts/)` —, et le § 8
de la spec exige qu'il vaille sur **deux appels séparés** à `lot`, pas sur une passe à deux
cibles : c'est précisément ce que le test existant couvre déjà et ce qu'il ne suffit pas à
prouver.

## 5. Ce que Supprimer doit effacer, et ce qu'il ne peut pas savoir

**5a. Cinq noms se reconstruisent de la clé, un est fixe.** `nom()` (`package.rs:62`) donne
`interieur-<clé>.typ`, `interieur-<clé>.pdf`, `couverture-<clé>.typ`, `couverture-<clé>.pdf`,
`couverture-<clé>.png` ; la fiche est `televersement.txt` (`package.rs:520`), sans clé, « il
n'y a qu'une fiche par répertoire ».

**5b. Les images de couverture sont écrites sous leur nom d'origine** (`ecrire_table`,
`package.rs:845-851`) : la seule liste qui les nomme est `projet.images`, c'est-à-dire l'état
**courant** du projet, pas ce que la génération avait écrit. Une image retirée du projet après
la génération survivra donc à la suppression et se nommera au compte rendu comme un fichier
étranger. C'est un moindre mal — la spec préfère laisser survivre que d'effacer au jugé — mais
le compte rendu ne doit pas l'annoncer comme un dépôt de l'utilisateur.

**5c. Le précédent d'effacement est `ebook::efface` (`ebook.rs:209`)**, et son arbitrage est
celui que la spec réclame : `NotFound` n'est pas une erreur, tout autre échec refuse plutôt
que de passer outre en silence.

**5d. Le répertoire de travail de `composer` ne peut pas être emporté.** `composer` range
sous la clé du **gabarit** (`commands.rs:793`, via `sorties_dossier(o, &pr.cle)`), à trois
segments (`catalogue.rs:915`) ; un livrable a quatre segments (`catalogue.rs:906`). Les deux
répertoires ne se confondent jamais, et Supprimer ne peut pas retirer sous les pieds de
l'onglet Intérieur le PDF qu'il affiche.

**5e. Un livrable dont `normalise` a replié le papier a changé de clé** (`projet.rs:538`),
donc de répertoire : les fichiers de l'ancienne clé lui deviennent étrangers et resteront.
C'était déjà vrai avant ce chantier — Retirer ne touchait à rien —, et rien ici ne l'aggrave.

## 6. Deux types nommés `Generation` vont se croiser dans `commands.rs`

`commands::Generation` (`commands.rs:1553`) est le compte rendu que `packager` rend au front ;
`projet::Generation` (`projet.rs:301`) est l'état qu'un livrable retient. `commands.rs`
n'importe aujourd'hui que le second groupe de types de `projet` (`commands.rs:24`), sans lui.
Le lot 2 manipulera les deux dans ce fichier : écrire `projet::Generation` en toutes lettres
suffit, et vaut mieux qu'un import qui ferait dépendre le sens de `Generation` de l'endroit
où on lit.

## 7. La vue ne peut pas encore porter l'état

`livraison_vue` (`commands.rs:2374`) ne reçoit que la `Livraison`, quand `empreinte::etat`
réclame le `Projet` entier — texte, livre, réglages, images. Faire descendre `Etat` dans
`LivrableVue` change donc la signature de son unique appelant de production (`commands.rs:2447`) et
celle du test qui l'éprouve (`commands.rs:2813`).
Ce n'est pas gratuit, et la spec range l'écran au lot 3. **Verdict** : le lot 2 écrit l'état
dans la donnée et n'y touche pas dans la vue ; le lot 3 fera descendre `Etat` avec le reste de
l'écran, en une fois.

## 8. Un écart entre la spec et le lot 1, sans conséquence pratique

La spec § 2 veut que « la péremption couvre tout ce qui entre dans les fichiers ». La finition
entre dans `televersement.txt` (`ecrire_fiche`, `package.rs:513-525`) et n'entre dans aucune
des deux empreintes (`empreinte.rs:33` et `58`). Un livrable dont on ne changerait que la
finition resterait donc marqué à jour devant une fiche périmée. Le trou ne s'ouvre pas :
après le lot 3, le seul chemin qui change une finition est Remplacer, qui recompose toujours.
**Verdict** : ne rien changer aux empreintes, et le dire — ajouter la finition à l'empreinte
de couverture périmerait la planche pour une donnée qui ne fabrique aucun de ses octets.
