# L'étape Livraison refondue : le livrable est son package

Date : 2026-08-29
Statut : validé (brainstorming)

## Objectif

L'étape Livraison a deux zones et un temps mort entre elles. On déclare des livrables dans
la première, on attend, et la seconde montre ce qu'ils ont produit. La zone haute n'affiche
pourtant rien qu'on ne sache déjà : le format et le fond perdu viennent du catalogue, la
pagination d'une composition passée. Elle occupe la moitié de la fenêtre pour répéter, ligne
après ligne, ce que le groupe entier a en commun — trois livrables TheBookEdition y écrivent
trois fois le même nom, la même reliure, le même pelliculage, la même pagination et le même
dos, pour ne différer que par le papier.

Cette spec fond les deux zones en une. **Un livrable naît généré** : le formulaire d'ajout
demande les cinq axes d'un coup et compose ; ce qui paraît dans la liste est le package, avec
sa vignette, ses chiffres et ses fichiers. Les livrables se rangent par imprimeur, et chacun
porte les gestes qui le concernent — modifier, dupliquer, régénérer, supprimer.

Elle ne touche ni à ce que l'application compose, ni à la pagination, ni au catalogue. Le
témoin doit valoir le même compte de pages à la fin qu'au début.

## Décisions de cadrage (brainstorming du 29/08)

- **Le formulaire porte deux verbes, la ligne n'a plus de contrôle.** Le même formulaire sert
  à créer et à modifier : ouvert depuis une ligne il est prérempli et son bouton dit
  « Remplacer », sinon il dit « Générer ». Écarté : garder les listes déroulantes sur chaque
  ligne — c'est justement ce qui la charge ; et l'immuabilité pure, qui ferait payer une
  garde et cinq champs pour corriger un papier.
- **La péremption couvre tout ce qui entre dans les fichiers**, couverture comprise. Le
  mécanisme actuel ne périme que la pagination ; une maquette retouchée laisserait à l'écran
  une planche fausse avec l'air d'être juste — et c'est elle qui porte le dos.
- **Supprimer efface ce que l'application a écrit**, puis retire le répertoire s'il ne reste
  rien. Un fichier étranger survit et se nomme au compte rendu. Écarté : l'effacement récursif
  sans condition, qui emporterait sans recours ce qu'on aurait déposé là.
- **Un échec de composition crée quand même le livrable.** Il paraît en erreur, avec son
  message et son bouton Régénérer. Sans quoi la seule issue serait de tout ressaisir.
- **La mutualisation de l'intérieur survit à la génération unitaire.** C'est la contrepartie
  de générer à l'ajout, et elle n'est pas négociable : trois papiers d'un même gabarit
  doivent coûter une composition et deux copies, qu'on les ajoute d'un coup ou un par un.

## 1. Ce qui change de sens

| terme | avant | après |
|---|---|---|
| livrable | une déclaration qu'on règle, puis qu'on compose | une configuration **et** son package |
| Générer les packages | le seul bouton, prend tout | « Tout regénérer », en tête d'étape |
| Retirer | ôte du projet, laisse les fichiers | remplacé par Supprimer, qui nettoie |
| compte rendu | une zone à part, sous la liste | le corps de chaque ligne |

La clé du livrable, à quatre axes, ne change pas : elle nomme toujours son répertoire et ses
fichiers. La clé du gabarit non plus — c'est elle qui range la mesure et la composition.

## 2. Le modèle

Un livrable gagne un **état de génération**, et rien d'autre :

```
jamais généré                                    (le cas d'un .ozalid d'avant, ou d'un ajout
                                                  dont la composition n'a pas encore tourné)
généré { empreinte_intérieur, empreinte_couverture }
en erreur { message }
```

L'état est optionnel dans le fichier : un `.ozalid` existant s'ouvre en *jamais généré*, ses
relevés intacts. Aucun format à casser, aucune migration à écrire.

### Les deux empreintes

**Deux et non une**, pour deux raisons qui se renforcent : dire *quoi* a bougé vaut mieux que
dire *quelque chose*, et réutiliser un intérieur suppose de savoir que la part intérieure n'a
pas bougé — même quand la couverture, elle, a bougé.

- **`empreinte_intérieur`** — le manuscrit, l'identité du livre (donc les liminaires et
  l'ISBN), les réglages d'intérieur, et l'empreinte du gabarit que `Resolu::empreinte` rend
  déjà.
- **`empreinte_couverture`** — la maquette, ses images, ses réglages, et le dos effectif
  (qui dépend du papier et de la pagination).

Ce qui n'y entre pas : les envois et l'épreuve de relecture. Ils ne touchent aucun octet de
ces deux PDF — les exemplaires dédicacés ont leurs propres répertoires.

Les mesures par gabarit (`Livraison::mesures`) restent ce qu'elles sont : elles servent le
pied « Vu pour » et la réutilisation de l'intérieur. Ce qui change, c'est que la péremption ne
s'affiche plus par effacement de mesure mais par **comparaison d'empreintes** : un package
périmé garde ses chiffres à l'écran, marqués pour ce qu'ils sont. `oublier_mesures` garde son
rôle sur la pagination.

## 3. Le cycle d'un livrable

- **Générer** — le formulaire pose le livrable (le refus de doublon sur les quatre axes ne
  change pas), puis compose. Cet ordre est ce qui donne une place à l'échec.
- **Modifier** — le formulaire s'ouvre prérempli, son bouton dit « Remplacer », un « Annuler »
  paraît, et la ligne visée est marquée le temps de l'édition. En ce mode, le livrable édité
  ne se compte pas comme doublon de lui-même. Rien n'interdit d'y changer l'imprimeur : le
  livrable quitte alors son groupe pour le nouveau, en queue, comme s'il venait d'y être
  ajouté — c'est un remplacement, pas un déménagement à part.
- **Dupliquer** — le même formulaire prérempli, mais en mode neutre : il crée.
- **Régénérer** — recompose sans toucher aux axes.
- **Supprimer** — garde en deux temps, comme tout ce qui défait dans l'application (le premier
  clic arme, le second retire). Puis les fichiers, puis le livrable.

### Remplacer compose avant d'effacer

Si la nouvelle composition échouait après qu'on a vidé l'ancien répertoire, on aurait échangé
un package qui marchait contre un qui ne marche pas. L'ordre est donc : composer dans le
nouveau répertoire, puis effacer l'ancien, et seulement en cas de succès. Quand les deux
portent la même clé — on n'a changé que la finition, qui ne fabrique rien —, il n'y a rien à
effacer.

### Ce que Supprimer efface

Les fichiers que l'application a écrits dans ce répertoire : les deux PDF, la vignette, les
deux sources Typst, l'image de couverture copiée, la fiche de téléversement. Puis le
répertoire lui-même **s'il est vide**. S'il reste autre chose, il survit et le compte rendu
le nomme. Un fichier déjà parti n'est pas une erreur : le livrable s'en va, et le compte rendu
dit ce qui n'était plus là.

## 4. La réutilisation de l'intérieur

`package::lot` sait déjà ne composer l'intérieur qu'une fois par gabarit à l'intérieur d'une
passe. Il apprend à **amorcer cette mémoire avec ce que le disque porte déjà** : un livrable
généré, du même gabarit, dont l'`empreinte_intérieur` est celle de l'état courant et dont le
PDF est là.

Générer un livrable devient alors `lot` avec une seule cible ; « Tout regénérer », `lot` avec
toutes. Un seul chemin de composition, un seul jeu de garanties. C'est ce qui fait que trois
papiers d'un même gabarit coûtent une composition et deux copies, qu'on les ajoute d'un coup
ou un par un.

## 5. L'écran

Une seule zone. Ce qui est aujourd'hui le compte rendu devient le corps de chaque ligne.

```
LIVRABLES
Chaque livrable compose son propre intérieur, donc sa propre pagination,
donc son propre dos et sa propre planche.

[Imprimeur ▾] [Format ▾] [Reliure ▾] [Pelliculage ▾] [Papier ▾]   [Générer]

                                                    [Tout regénérer]

Lulu
    Poche 10,8 × 17,5 — Broché — Brillant — Crème non couché 60 lb
    266 p · gouttière 25,4 mm · dos 16,74 mm · FP 3,175 mm
    ┌────────┐  interieur-lulu-….pdf   couverture-lulu-….pdf
    │vignette│  televersement.txt
    └────────┘  /Users/…/LHC/lulu-108x175-broche-creme-60/
    ✎ Modifier   ⧉ Dupliquer   ⟳ Régénérer   ⌫ Supprimer

TheBookEdition
    Poche 11 × 17 — Broché — Brillant — Papier 135 g couleur
    ⚠ la couverture a changé depuis cette génération
    …
```

- **Le groupe porte l'imprimeur, la ligne ne le répète plus.** C'est ce qui règle la
  répétition constatée : trois TheBookEdition ne diffèrent plus à l'écran que par ce qui les
  distingue vraiment.
- **Les groupes se rangent dans l'ordre du premier ajout**, et les lignes dans leur groupe de
  même. Un ordre stable, qui ne se réarrange pas sous la main.
- **Les groupes ne se replient pas.** Un imprimeur porte deux ou trois livrables, pas trente ;
  un pli serait un état de plus à tenir pour un gain qu'on ne mesure pas.
- **« Tout regénérer » est global**, en tête d'étape. Un bouton par groupe serait un troisième
  verbe à expliquer.
- **Les relevés entrent dans le formulaire**, sous les cinq listes, et seulement si
  l'imprimeur choisi en exige. Aucun des six catalogues fournis n'est dans ce cas — c'est un
  fichier déposé sur le poste qui les fait paraître.
- **L'attente** garde son dispositif : bouton éteint et ligne d'état, comme `packager()` le
  fait déjà.
- **Les alertes descendent sur la ligne** qu'elles concernent — police introuvable, dos rogné
  au pli. Elles seront plus près de leur objet qu'aujourd'hui.

Ne bougent pas : le pied « Vu pour », la génération d'ebooks, la vignette et sa largeur.

## 6. Les commandes

| commande | rôle |
|---|---|
| `livrable_generer` | pose le livrable, puis compose |
| `livrable_remplacer` | même chose sur un livrable existant, à sa place |
| `livrable_regenerer` | recompose sans toucher aux axes |
| `livrable_supprimer` | efface les fichiers connus, retire le répertoire s'il est vide, retire le livrable |
| `packager` | inchangée dans son principe — c'est « Tout regénérer » |
| `livrable_viser` | inchangée (le pied, les ebooks) |

`livrable_ajouter`, `livrable_regler` et `livrable_retirer` disparaissent : leurs gestes
n'existent plus.

## 7. Ce qui bouge ailleurs

- **Le README** : la section « 3 · Livraison » décrit le geste en deux temps. Elle est à
  réécrire sur le nouveau.
- **Les `.ozalid` existants** s'ouvrent sans conversion, leurs livrables en *jamais généré*.
- **Le fichier de sortie** ne change pas : mêmes noms, même répertoire par livrable, même
  fiche de téléversement.

## 8. Risques

- **Le temps d'attente se déplace.** Il était concentré sur un bouton, il se répartit sur
  chaque ajout. La réutilisation de l'intérieur (§ 4) est ce qui rend ça tenable ; si elle
  était mal câblée, le symptôme serait un ajout lent sans que rien ne le dise. Le test qui
  l'ancre doit valoir sur deux générations **séparées**, pas seulement dans une passe.
- **Les empreintes peuvent périmer trop, ou trop peu.** Trop : la liste crie au loup et on
  cesse de la lire. Trop peu : un dos faux part chez l'imprimeur. Les tests doivent tenir les
  deux bords, champ par champ.
- **Remplacer touche au disque.** L'ordre composer-puis-effacer est ce qui protège ; il doit
  être éprouvé sur l'échec, pas seulement sur le succès.

## 9. Vérification

### Le témoin

`cargo run --example temoin` doit valoir exactement le même compte de pages qu'avant le
chantier : rien ici ne touche la pagination. Un écart est une régression, jamais un effet de
bord admis.

### Ce que les tests doivent tenir

- Un intérieur composé pour un livrable est **copié**, non recomposé, par un livrable du même
  gabarit généré dans un **appel séparé**.
- Une empreinte d'intérieur inchangée mais une couverture retouchée périme la couverture,
  et elle seule ; l'inverse aussi.
- Un envoi ajouté ou une épreuve tirée ne périment rien.
- Supprimer efface les fichiers connus, conserve un fichier étranger et le nomme.
- Supprimer sur un répertoire déjà parti retire le livrable sans échouer.
- Remplacer dont la composition échoue laisse l'ancien package intact.
- Remplacer ne se refuse pas lui-même au titre du doublon.
- Un `.ozalid` d'avant s'ouvre, ses livrables en *jamais généré*, ses relevés intacts.

Chaque test neuf doit avoir été **vu échouer** avant d'être vert.

### À l'œil

Ce que le faux DOM ne peut pas dire : que la liste tient dans la fenêtre, que le groupe se lit
comme un groupe, et que le marquage de péremption se voit sans être criard. À regarder sur le
projet réel, aux deux largeurs de fenêtre.

## 10. Les lots

1. **Le modèle** — état de génération, les deux empreintes, l'ouverture d'un `.ozalid` d'avant.
   Aucun changement visible ; tout se vérifie en tests Rust.
2. **Les commandes** — les quatre verbes, la réutilisation d'intérieur depuis le disque, la
   suppression sélective. L'écran actuel continue de tourner.
3. **L'écran** — le formulaire à deux verbes, le groupement par imprimeur, les quatre boutons,
   la disparition de la zone intermédiaire, le README.

## Hors périmètre

- Composer une reliure que l'application n'outille pas. Le grisé reste, sa réserve est au
  README depuis le 29/08.
- Toucher au catalogue, aux formules de dos ou aux relevés d'imprimeur.
- Les ebooks, qui gardent leur bouton et leur dépendance au livrable visé.
- Les exemplaires dédicacés de l'étape Envois : ils ont leurs propres répertoires, leur propre
  bouton, et rien ici ne les touche.
- Toute optimisation de la mise en page au-delà de ce que le groupement rend : les pistes
  restantes (bases de `.livrable .nom` et `.note`, paragraphe d'en-tête) sont à reprendre après.
