'use strict';

/**
 * L'étape Livraison : les livrables et leurs packages. Les envois ont leur
 * étape et leur fichier, `envois.js`.
 *
 * Même partage que `couverture.js` : ce fichier ne pose aucun écouteur et ne lit pas
 * le DOM au chargement. Il définit, `app.js` branche — c'est ce qui permet aux deux de
 * vivre dans le même contexte global sans dépendre de l'ordre de chargement.
 */

/**
 * Les fichiers de catalogue du poste que le démarrage n'a pas pu lire.
 *
 * Muet sur un poste qui n'en dépose aucun — le cas de presque tous. Quand il parle, il
 * nomme le fichier et la raison : un POD absent de la liste sans explication se lirait
 * comme un POD qui n'existe pas, et la faute se chercherait dans le fichier plutôt que
 * dans sa syntaxe.
 */
async function afficherRefusCatalogue() {
  const refus = await invoke('catalogue_refus');
  const box = $('refusCatalogue');
  box.hidden = refus.length === 0;
  box.replaceChildren();
  for (const r of refus) {
    // Le nom d'abord, jamais tronqué, son répertoire ensuite et le chemin entier au
    // survol : ce découpage est celui des projets récents, et pour la même raison —
    // ces fichiers sortent tous du même `pods/`, et coupés par la fin ils se liraient
    // tous pareil. Le montage, lui, est celui de l'entête : une seule ligne où le
    // répertoire prend la place qui reste, là où les récents empilent. Les deux
    // séparateurs, comme les récents : l'application est aussi empaquetée pour Windows.
    const coupe = Math.max(r.fichier.lastIndexOf('/'), r.fichier.lastIndexOf('\\'));
    const fichier = h('span', undefined, 'fichier');
    fichier.title = r.fichier;
    fichier.append(
      h('span', `Catalogue non chargé : ${r.fichier.slice(coupe + 1)}`),
      h('span', r.fichier.slice(0, Math.max(coupe, 0)), 'chemin')
    );
    // La raison sur sa propre ligne : c'est la seule partie du message qui dit quoi
    // corriger, et elle doit se lire jusqu'au bout même quand elle fait trois phrases.
    const ligne = h('p', undefined, 'note alerte');
    ligne.append(fichier, h('span', r.raison));
    box.append(ligne);
  }
  if (refus.length) {
    // Les deux choses que l'utilisateur ne peut pas déduire de l'écran : que la liste
    // ci-dessous est complète de ce qui a pu être chargé, et qu'un fichier corrigé ne
    // sera relu qu'au prochain démarrage.
    box.append(h('p', 'Les POD fournis, eux, sont là. Corriger '
      + `${refus.length > 1 ? 'ces fichiers' : 'ce fichier'}, puis relancer : ceux du `
      + 'poste ne sont lus qu\'au démarrage.', 'note'));
  }
}

/**
 * Ce que l'ouverture a retiré de la liste.
 *
 * Même traitement que les fichiers de catalogue refusés, et à la suite : on ne peut pas
 * rétablir un livrable dont le catalogue ne porte plus l'axe, mais on peut dire lequel.
 * Sans ce mot, un livrable réglé disparaît entre deux ouvertures et le livre paraît
 * s'être défait tout seul.
 *
 * Muet quand rien n'a été retiré — presque toujours : une boîte qui s'afficherait vide
 * à chaque ouverture serait pire que le silence qu'elle corrige.
 */
function majElagues(vue) {
  const partis = vue.elagues ?? [];
  const box = $('livrablesElagues');
  box.hidden = partis.length === 0;
  // La cause dans la phrase, comme les refus la donnent : c'est elle qui dit où aller
  // corriger — un fichier de `pods/` du poste, ou un livre venu d'une machine mieux
  // pourvue. Sans elle, la disparition se lirait comme une perte du fichier lui-même.
  //
  // La clé brute, et non un libellé, et ce n'est pas un niveau de langage à améliorer :
  // c'est le seul qui reste. Ces clés portent quatre axes quand `libelleProvider` en
  // attend trois, et lui il retombe **de toute façon** sur la clé pour un gabarit que la
  // table plate ne connaît plus — c'est exactement le cas de ceux-ci. Un nom complet est
  // impossible par construction : ce qui le porterait est précisément ce qui a disparu.
  //
  // Et ce qu'on peut faire, à la suite : les refus disent « corriger, puis relancer »,
  // sans quoi le message ne serait qu'un constat de perte. Ici l'axe manquant peut être
  // n'importe lequel des quatre, la phrase dit donc « le catalogue » et non l'imprimeur.
  const pluriel = partis.length > 1;
  box.textContent = box.hidden
    ? ''
    : `${pluriel ? 'Livrables retirés' : 'Livrable retiré'} à l'ouverture, faute de `
      + `catalogue qui ${pluriel ? 'les porte' : 'le porte'} encore : ${partis.join(', ')}. `
      + `Le reste du livre est intact : ${pluriel ? 'les rajouter' : 'le rajouter'} `
      + 'ci-dessous une fois le catalogue complété.';
}

/**
 * Ce que la composition a vu, par clé de livrable, pour la durée de la session.
 *
 * Le `.ozalid` retient la mesure et les deux empreintes, jamais le dos rogné, les
 * avertissements ni le partage d'intérieur : ceux-là ne se reconstruisent pas sans
 * composer. Ils vivent donc ici, et une ligne rouverte demain se tait là-dessus plutôt que
 * d'inventer un silence rassurant sur un fichier qu'elle n'a pas vu naître.
 */
let packagesDeLaSession = {};

/** Range les comptes rendus d'une composition, puis réaffiche les lignes qui les portent. */
function retenirPackagesDeLaSession(resultats) {
  for (const r of resultats ?? []) packagesDeLaSession[r.cle] = r;
  afficherLivrables();
}

/**
 * Oublie ce que la session avait vu composer.
 *
 * Indispensable au changement de projet : ces comptes rendus sont rangés **par clé de
 * livrable**, et deux livres tirés chez le même imprimeur, au même format et sur le même
 * papier portent la même clé. Sans cet oubli, la ligne d'un livre neuf afficherait le dos,
 * les chemins et les alertes du livre qu'on vient de fermer.
 */
function oublierPackagesDeLaSession() {
  packagesDeLaSession = {};
}

/**
 * Les livrables du livre, groupés par imprimeur.
 *
 * Le groupe porte l'imprimeur, la ligne ne le répète plus : trois livrables du même POD ne
 * diffèrent plus à l'écran que par ce qui les distingue vraiment. Les groupes se rangent
 * dans l'ordre du premier ajout, les lignes dans leur groupe de même — un ordre stable,
 * qui ne se réarrange pas sous la main, parce qu'un ordre qui bouge fait perdre la ligne
 * qu'on visait entre deux clics.
 *
 * Les groupes ne se replient pas : un imprimeur porte deux ou trois livrables, pas trente,
 * et un pli serait un état de plus à tenir pour un gain qu'on ne mesure pas.
 */
function afficherLivrables() {
  const box = $('livrables');
  box.replaceChildren();
  // L'ordre du premier ajout, sans tri : `Map` garde l'ordre d'insertion, et le premier
  // livrable d'un POD est celui qui pose son groupe.
  const groupes = new Map();
  for (const d of projet.livraison.livrables) {
    if (!groupes.has(d.pod)) groupes.set(d.pod, []);
    groupes.get(d.pod).push(d);
  }
  for (const [pod, lignes] of groupes) {
    const bloc = h('div', undefined, 'groupe');
    bloc.id = `groupe-${pod}`;
    bloc.append(h('h3', pods.find((p) => p.cle === pod)?.nom ?? pod));
    for (const d of lignes) bloc.append(ligneLivrable(d));
    box.append(bloc);
  }

  afficherFormulaire();
  chargerVignettes();
}

/**
 * Une ligne : ce qu'on sait de ce livrable, et les gestes qu'on peut lui faire.
 *
 * Deux niveaux de remplissage, et c'est voulu. Ce que le modèle retient — identité, pages,
 * gouttière, dos, état — se lit toujours, y compris à la réouverture d'un projet fermé la
 * veille. Ce que seule la composition a vu — dos rogné, avertissements, polices de repli —
 * ne paraît que dans la session qui a généré.
 */
function ligneLivrable(d) {
  const ligne = h('div', undefined, 'livrable');
  ligne.id = `liv-${d.cle}`;
  const p = providers.find((pr) => pr.cle === d.gabarit);
  const pod = pods.find((x) => x.cle === d.pod);
  const dosPublie = pod?.papiers.find((pa) => pa.cle === d.papier)?.dos_publie ?? false;

  const infos = h('div', undefined, 'infos');
  infos.append(h('span', libelleDansGroupe(d), 'nom'));
  if (p) infos.append(h('p', noteFormat(p), 'note'));
  infos.append(noteMesure(d, dosPublie), noteEtat(d));

  const r = packagesDeLaSession[d.cle];
  if (r?.package) {
    const q = r.package;
    const dl = h('dl');
    for (const [k, v] of [
      ['Pages', `${q.pages}${q.blanche ? ' (blanche de parité)' : ''}`],
      ['Papier', q.papier],
      // Après le papier — l'autre chose qu'on choisit sans qu'un octet du PDF change, et
      // qui se commande quand même. Elle ne paraît que là où il y en a une.
      ...(r.finition ? [['Finition', r.finition]] : []),
      ['Gouttière', `${nb(q.gouttiere, 1)} mm`],
      ['Dos', `${nb(q.dos)} mm`],
      ['Planche', `${nb(q.planche[0])} × ${nb(q.planche[1])} mm, `
        + `FP ${nb(q.fond_perdu, 3)} mm`],
    ]) dl.append(h('dt', k), h('dd', v));
    infos.append(dl);
    // Le dos est composé sur une zone qui rogne ce qui dépasse, sans rien dire : un titre
    // coupé au pli ne se verrait qu'à l'impression.
    if (q.dos_requis !== null) {
      infos.append(h('p', `Dos de ${nb(q.dos)} mm pour un texte qui en réclame `
        + `${nb(q.dos_requis)} mm : il sera rogné au pli. Réduire le corps du dos, ou `
        + 'y éteindre un élément.', 'note alerte'));
    }
    // Une police que Typst a remplacée sans échouer : ce PDF-là part chez l'imprimeur.
    if (q.polices_introuvables.length) {
      infos.append(h('p', 'Police introuvable, composé dans une écriture de repli : '
        + `${q.polices_introuvables.join(', ')}. Le PDF ne suit pas la maquette.`,
      'note alerte'));
    }
    // En gris et non en rouge, à la différence des deux alertes ci-dessus : celles-là
    // disent qu'un PDF ne suit pas la maquette, celles-ci qu'un tirage juste ne plaira
    // peut-être pas. C'est un jugement d'auteur, et le rouge perdrait son sens à couvrir
    // les deux. Les phrases viennent du Rust telles quelles : la fiche de téléversement
    // les recopie, et un dossier relu trois mois plus tard doit dire ce que l'écran disait.
    for (const a of q.avertissements) infos.append(h('p', a, 'note'));
    for (const c of cheminsGroupes(q.chemins)) infos.append(h('p', c, 'chemin'));
  } else if (r?.erreur) {
    infos.append(h('p', r.erreur, 'note alerte'));
  }

  // La planche telle qu'elle part à l'impression, avec le dos mesuré de ce livrable-là :
  // c'est ici que « est-ce que ça tient » se vérifie, sur du vrai. La source arrive du
  // compte rendu de la session ou, à la réouverture, de `chargerVignettes`.
  const img = h('img', undefined, 'vignette');
  img.id = `liv-vignette-${d.cle}`;
  img.alt = `Planche composée pour ${libelleDansGroupe(d)}`;
  if (r?.vignette) img.src = r.vignette;

  ligne.append(infos, img, gestesLivrable(d));
  return ligne;
}

/**
 * Le livrable en cours de modification, ou `null` quand le formulaire ajoute.
 *
 * C'est ce qui donne au bouton son second verbe. Une modification abandonnée — on clique
 * Modifier puis on change d'avis — se défait en cliquant Dupliquer, ou en modifiant une
 * autre ligne : le formulaire n'a qu'un état, et il est toujours celui du dernier geste.
 */
let remplace = null;

/**
 * Les quatre verbes d'un livrable, dans l'ordre du geste : on modifie plus souvent qu'on ne
 * duplique, on duplique plus souvent qu'on ne régénère, et on supprime en dernier. Ce qui
 * défait est à droite, comme le retrait l'était.
 */
function gestesLivrable(d) {
  const bouton = (quoi, texte, ecoute) => {
    const b = h('button', texte);
    b.type = 'button';
    b.className = 'nu';
    b.id = `liv-${quoi}-${d.cle}`;
    b.addEventListener('click', ecoute);
    return b;
  };
  const gestes = h('div', undefined, 'gestes');
  // Supprimer emporte la ligne, son package et les relevés qu'on y a saisis, sans reprise,
  // au milieu de trois gestes qu'on fait couramment : le premier clic arme, le second
  // supprime. Même dispositif que le retrait d'avant le lot 3 et que l'effacement d'une
  // maquette — et la raison s'est aggravée, puisque le geste emporte maintenant les
  // fichiers avec la ligne.
  const supprimer = bouton('supprimer', '⌫ Supprimer', () => {
    if (armeSur(supprimer)) {
      desarmerGeste();
      return supprimerLivrable(d);
    }
    armerGeste(supprimer, () => {
      supprimer.textContent = '⌫ Supprimer';
      supprimer.className = 'nu';
    });
    supprimer.textContent = 'Confirmer';
    supprimer.className = 'danger';
    return undefined;
  });
  // Le dernier ne se supprime pas : c'est lui qui donne le format sous lequel on regarde
  // la couverture. Le Rust refuse, mais un bouton qui ne peut qu'échouer vaut mieux éteint
  // que refusé.
  supprimer.disabled = projet.livraison.livrables.length < 2;
  gestes.append(
    bouton('modifier', '✎ Modifier', () => ouvrirModification(d)),
    bouton('dupliquer', '⧉ Dupliquer', () => ouvrirDuplication(d)),
    bouton('regenerer', '⟳ Régénérer', () => regenererLivrable(d)),
    supprimer,
  );
  return gestes;
}

/** Modifier : le formulaire reprend cette ligne, et son verbe devient Remplacer. */
function ouvrirModification(d) {
  remplace = d.cle;
  remplirFormulaire(d);
  $('btLivrableGenerer').textContent = 'Remplacer';
}

/** Dupliquer : les mêmes axes, mais c'est un ajout — le geste qui compare deux papiers. */
function ouvrirDuplication(d) {
  remplace = null;
  remplirFormulaire(d);
  $('btLivrableGenerer').textContent = 'Générer';
}

/**
 * Remplit le formulaire avec les axes d'un livrable existant.
 *
 * L'ordre n'est pas négociable : le POD d'abord, puis `afficherAxesDuPod`, **puis** les
 * quatre autres valeurs — les listes de format, reliure, papier et pelliculage n'existent
 * qu'après, et poser une valeur dans une liste vide la perd sans rien dire.
 *
 * Les deux relevés sont repris, et c'est la raison d'être de Modifier : depuis que la ligne
 * ne porte plus de contrôle, c'est le seul chemin par lequel un dos relevé se corrige. Les
 * oublier ferait de la correction d'un chiffre une ressaisie complète.
 */
function remplirFormulaire(d) {
  $('inAjoutPod').value = d.pod;
  afficherAxesDuPod();
  $('inAjoutFormat').value = d.format;
  $('inAjoutReliure').value = d.reliure;
  $('inAjoutPapier').value = d.papier;
  if ($('inAjoutFinition')) $('inAjoutFinition').value = d.finition ?? '';
  // Les relevés dépendent du papier, qui vient d'être posé : les champs n'existent qu'une
  // fois `afficherRelevesDuFormulaire` rappelée avec ce papier-là.
  afficherRelevesDuFormulaire(pods.find((x) => x.cle === d.pod));
  if ($('inAjoutDos')) $('inAjoutDos').value = d.dos_mm ?? '';
  if ($('inAjoutFp')) $('inAjoutFp').value = d.fond_perdu_mm ?? '';
}

/**
 * Où en est le package de cette ligne, en une phrase.
 *
 * La péremption dit **ce qui** a bougé : « périmé » tout court obligerait à régénérer pour
 * apprendre si c'est le texte ou la maquette, et les deux ne coûtent pas la même chose à
 * recomposer. L'échec dit sa raison, pour la même raison — l'apprendre autrement
 * demanderait de refaire la composition qui a échoué.
 *
 * Jamais généré n'est pas une alerte : ce livrable n'a rien perdu, on ne lui a rien demandé.
 */
function noteEtat(d) {
  const p = h('p', undefined, 'note');
  p.id = `liv-etat-${d.cle}`;
  const e = d.etat ?? { etat: 'jamais' };
  if (e.etat === 'jamais') {
    p.textContent = 'jamais généré';
  } else if (e.etat === 'ajour') {
    p.textContent = 'à jour';
  } else if (e.etat === 'echec') {
    p.className = 'note alerte';
    p.textContent = `la dernière génération a échoué : ${e.message}`;
  } else {
    p.className = 'note alerte';
    const quoi = [
      ...(e.interieur ? ['le texte'] : []),
      ...(e.couverture ? ['la couverture'] : []),
    ].join(' et ');
    p.textContent = `${quoi} a changé depuis cette génération`;
  }
  return p;
}

/**
 * Les vignettes laissées sur le disque par les générations d'avant.
 *
 * Hors de la vue et après le montage des lignes : `livraison_vue` est rendue par toute
 * commande qui écrit, et un base64 par livrable à chaque frappe se paierait pour rien. La
 * ligne se monte sans, et la vignette s'y pose quand elle arrive.
 */
async function chargerVignettes() {
  const table = await invoke('livrable_vignettes');
  for (const [cle, donnee] of Object.entries(table)) {
    const img = $(`liv-vignette-${cle}`);
    // Seulement si la session n'en a pas de plus fraîche : celle qu'on vient de composer
    // est celle du fichier qui vient d'être écrit.
    if (img && !img.src) img.src = donnee;
  }
}

/**
 * Le formulaire d'un livrable : les cinq axes, puis les relevés que l'imprimeur exige.
 *
 * Aucun filtre sur ce qui est déjà déclaré : c'est ce qui permet de déclarer deux fois le
 * même gabarit pour comparer deux papiers. Le vrai doublon — les quatre axes identiques —
 * est refusé par le Rust, avec sa raison.
 *
 * Les listes se reconstruisent à chaque affichage : elles ne dépendent que du catalogue,
 * qui ne bouge pas de la vie du processus, mais les reconstruire coûte cinq boucles sur
 * quelques entrées et évite d'avoir à se demander qui les a laissées dans quel état.
 */
function afficherFormulaire() {
  const sel = $('inAjoutPod');
  const choisi = sel.value;
  sel.replaceChildren();
  for (const p of pods) sel.append(new Option(p.nom, p.cle));
  // Le POD retenu survit à un réaffichage : générer un livrable ne doit pas ramener la
  // liste sur son premier, alors qu'on en ajoute souvent deux de suite chez le même.
  if (pods.some((p) => p.cle === choisi)) sel.value = choisi;
  sel.disabled = pods.length === 0;
  $('btLivrableGenerer').disabled = pods.length === 0;
  afficherAxesDuPod();
}

/**
 * Les quatre axes qui dépendent du POD choisi, et les relevés qui dépendent du papier.
 *
 * Chaque liste garde sa valeur si le POD neuf la porte encore, et la perd sinon : changer
 * de POD emporte de lui-même un format que le nouveau ne connaît pas. C'est ce qui laisse
 * intact le geste pour lequel cet écran existe — déclarer deux fois le même couple
 * imprimeur × format, puis changer le papier sur l'un des deux.
 *
 * La reliure non outillée reste **visible et grisée** : le Rust la refuse déjà en citant
 * sa raison (`catalogue::resout`), et l'écran ne fait que rendre ce refus lisible avant le
 * clic. Le grisé ne se glose pas — la réserve est au README, « Limites connues » : c'est
 * une limite de l'application, pas un fait du livrable.
 */
function afficherAxesDuPod() {
  const p = pods.find((x) => x.cle === $('inAjoutPod').value);
  const garde = (sel, valeurs) => {
    const choisi = sel.value;
    sel.replaceChildren();
    for (const [cle, nom, grise] of valeurs) {
      const o = new Option(nom, cle);
      o.disabled = grise;
      sel.append(o);
    }
    // La valeur retenue si le POD la porte encore ; sinon la première **composable**, et
    // non la première tout court. Un select se pose d'office sur sa première option, et
    // celle de KDP est une reliure grisée : le formulaire proposerait alors, dès son
    // ouverture, exactement ce que le Rust refuse en citant sa raison.
    const composable = valeurs.find(([, , grise]) => !grise);
    if (valeurs.some(([c]) => c === choisi)) sel.value = choisi;
    else if (composable) sel.value = composable[0];
    // Éteint seulement quand il n'y a qu'une valeur, toutes confondues : un select éteint
    // ne s'ouvre pas, et l'éteindre dès qu'il n'y a qu'une valeur composable cacherait
    // justement le grisé qu'on vient de poser.
    sel.disabled = valeurs.length < 2;
  };
  garde($('inAjoutFormat'), (p?.formats ?? []).map((f) => [f.cle, f.nom, false]));
  garde($('inAjoutReliure'),
    (p?.reliures ?? []).map((r) => [r.cle, r.nom, r.non_outille !== null]));
  garde($('inAjoutPapier'), (p?.papiers ?? []).map((pa) => [pa.cle, pa.libelle, false]));
  afficherFinition(p);
  afficherRelevesDuFormulaire(p);
}

/**
 * Le pelliculage, s'il y en a.
 *
 * Absent du DOM chez un POD qui n'en déclare aucun : un contrôle vide se lit comme un
 * choix qu'on n'a pas su faire, alors qu'il n'y en avait aucun à faire. Cinq des six POD
 * fournis sont dans ce cas.
 */
function afficherFinition(p) {
  const box = $('ajoutFinition');
  const choisi = $('inAjoutFinition')?.value;
  box.replaceChildren();
  if (!p?.finitions.length) return;
  const sel = h('select');
  sel.id = 'inAjoutFinition';
  sel.setAttribute('aria-label', 'Pelliculage');
  // Le vide en tête : aucune finition est le cas courant, et il doit rester choisissable
  // après en avoir pris une.
  sel.append(new Option('—', ''));
  for (const fi of p.finitions) sel.append(new Option(fi.nom, fi.cle));
  if (p.finitions.some((fi) => fi.cle === choisi)) sel.value = choisi;
  box.append(sel);
}

/**
 * Les relevés que l'imprimeur exige, sous les cinq listes.
 *
 * Le dos se réclame d'après **le papier retenu**, jamais d'après le POD : un POD peut
 * publier une formule pour l'un de ses papiers et pas pour l'autre. Le fond perdu, lui,
 * suit le gabarit — c'est la table plate qui seule sait le dire.
 *
 * Aucun des six POD fournis n'en exige : ce bloc reste vide sur un poste ordinaire, et ne
 * paraît que pour un catalogue déposé à la main.
 */
function afficherRelevesDuFormulaire(p) {
  const box = $('ajoutReleves');
  box.replaceChildren();
  const papier = p?.papiers.find((pa) => pa.cle === $('inAjoutPapier').value);
  const gabarit = providers.find((x) => x.cle === `${$('inAjoutPod').value}`
    + `-${$('inAjoutFormat').value}-${$('inAjoutReliure').value}`);
  if (papier && !papier.dos_publie) {
    box.append(champReleve('inAjoutDos', 'Dos relevé (mm)', null));
  }
  if (gabarit && gabarit.fond_perdu === null) {
    box.append(champReleve('inAjoutFp', 'FP (mm)', null));
  }
}

/**
 * Le livrable que le formulaire décrit, dans la forme exacte que les verbes attendent.
 *
 * Un champ vide est une absence de relevé, pas un zéro : composer sur un dos nul
 * produirait une planche fausse au lieu d'un refus. Un contrôle absent — le pelliculage
 * chez un POD qui n'en déclare aucun — vaut `null`, jamais une chaîne vide.
 */
function lireFormulaire() {
  const lu = (id) => {
    const v = $(id)?.value.trim();
    return v ? Number(v) : null;
  };
  return {
    pod: $('inAjoutPod').value,
    format: $('inAjoutFormat').value,
    reliure: $('inAjoutReliure').value,
    papier: $('inAjoutPapier').value,
    // La chaîne vide du choix « — » est une absence, pas une finition nommée.
    finition: $('inAjoutFinition')?.value || null,
    dos_mm: lu('inAjoutDos'),
    fond_perdu_mm: lu('inAjoutFp'),
  };
}

/**
 * Générer : pose le livrable et compose, d'un seul geste.
 *
 * L'attente garde le dispositif de `packager` — bouton éteint et ligne d'état — parce que
 * le temps de composition ne disparaît pas, il se répartit sur chaque ajout (spec § 8).
 */
async function pendantQueCaCompose(bt, mot, geste) {
  bt.disabled = true;
  $('etatLivraison').className = 'etat';
  $('etatLivraison').textContent = mot;
  try {
    const r = await geste();
    afficherProjet(r.projet);
    retenirPackagesDeLaSession(r.packages);
    $('etatLivraison').textContent = '';
    return r;
  } catch (e) {
    $('etatLivraison').textContent = String(e);
    $('etatLivraison').className = 'etat erreur';
    return null;
  } finally {
    bt.disabled = false;
  }
}

/**
 * Générer, ou Remplacer quand une ligne est en cours de modification.
 *
 * Un seul bouton pour les deux verbes, et son libellé dit lequel : le formulaire est le
 * même, et en ouvrir un second pour la modification obligerait à tenir deux jeux de
 * contrôles d'accord.
 */
async function genererLivrable() {
  const cle = remplace;
  const r = await pendantQueCaCompose(
    $('btLivrableGenerer'),
    'composition du package…',
    () => (cle === null
      ? invoke('livrable_generer', { livrable: lireFormulaire() })
      : invoke('livrable_remplacer', { cle, livrable: lireFormulaire() }))
  );
  if (r === null) return;
  remplace = null;
  $('btLivrableGenerer').textContent = 'Générer';
  // Ce que l'effacement de l'ancien répertoire n'a pas pu faire : la composition a réussi
  // et le projet porte le livrable neuf, mais un répertoire est resté. Sans ce mot, il
  // survit en silence — et l'on retrouve deux répertoires pour un livrable, sans savoir
  // lequel est parti chez l'imprimeur.
  if (r.nettoyage_echoue) {
    $('etatLivraison').textContent = r.nettoyage_echoue;
    $('etatLivraison').className = 'etat erreur';
  }
}

/**
 * Régénérer : recompose sans toucher aux axes.
 *
 * Peut légitimement **copier** l'intérieur d'un livrable du même gabarit déjà à jour au
 * lieu de le recomposer : c'est ce qui rend la comparaison de deux papiers gratuite. Seul
 * « Tout regénérer » recompose toujours.
 */
function regenererLivrable(d) {
  return pendantQueCaCompose(
    $(`liv-regenerer-${d.cle}`),
    'recomposition du package…',
    () => invoke('livrable_regenerer', { cle: d.cle })
  );
}

/**
 * Supprimer : efface les fichiers connus, retire le répertoire s'il est vide, retire le
 * livrable.
 *
 * Ce qui restait et que l'application n'a pas écrit **survit et se nomme** : le répertoire
 * reste pour lui. Le taire laisserait croire à un effacement complet, et l'on chercherait
 * six mois plus tard pourquoi le répertoire d'un livrable disparu traîne encore.
 */
async function supprimerLivrable(d) {
  await tente(async () => {
    const r = await invoke('livrable_supprimer', { cle: d.cle });
    afficherProjet(r.projet);
    if (r.nettoyage.etrangers.length) {
      $('etatLivraison').className = 'etat';
      $('etatLivraison').textContent = 'Le répertoire survit pour ce que l\'application '
        + `n'a pas écrit : ${r.nettoyage.etrangers.join(', ')}.`;
    }
  });
}

/**
 * Ce que la composition a mesuré pour ce livrable, ou pourquoi elle ne l'a pas mesuré.
 *
 * Un rang à soi sous la ligne, et non un ajout à la note du format : le format et le
 * fond perdu viennent du catalogue — connus sans rien composer, jamais périmés —, quand
 * pages, gouttière et dos viennent d'une composition qui n'a pas forcément eu lieu. Les
 * coudre dans la même phrase donnerait à lire comme également su ce qui ne l'est pas.
 *
 * Les décimales sont celles du compte rendu de package, plus bas dans le même onglet :
 * gouttière au dixième, dos au centième. Le pied fait l'inverse — un écart qui lui
 * appartient, et que ce lot ne corrige pas.
 */
function noteMesure(d, dosPublie) {
  const ligne = h('p', undefined, 'note mesure');
  ligne.id = `liv-mesure-${d.cle}`;
  if (!d.compose) {
    // Plus de nuance à faire ici depuis le lot 3 : « périmé » se dit sur la ligne d'état,
    // livrable par livrable et non pour toute la liste à la fois. Cette note-ci ne parle
    // plus que de la mesure, qu'on a ou qu'on n'a pas.
    ligne.textContent = 'non composé';
    return ligne;
  }
  // Le dos calculé là où le papier publie sa formule ; ailleurs le relevé fait sur le
  // gabarit, nommé comme tel — sans quoi il se lirait comme un chiffre que l'application
  // aurait trouvé seule. Et rien du tout si rien n'a été relevé : un dos absent ne
  // devient pas zéro parce que la pagination, elle, est connue.
  const dos = dosPublie ? d.compose.dos : d.dos_mm;
  ligne.textContent = [
    `${d.compose.pages} pages`,
    `gouttière ${nb(d.compose.gouttiere, 1)} mm`,
    ...(dos === null || dos === undefined
      ? []
      : [`dos ${nb(dos)} mm${dosPublie ? '' : ' (relevé)'}`]),
  ].join(' · ');
  return ligne;
}

function noteFormat(p) {
  // « FP » et non « fond perdu » : le terme entier tenait la moitié de la note et la
  // repliait, en emportant le « mm » de l'unité au rang suivant. Abrégé partout sur cet
  // onglet — champ de relevé compris —, jamais ailleurs : la Couverture, où le fond
  // perdu est ce qu'on règle, l'écrit en toutes lettres.
  const fp = p.fond_perdu === null
    ? 'FP à relever sur le gabarit'
    : `FP ${nb(p.fond_perdu, 3)} mm`;
  return `${nb(p.largeur, 1)} × ${nb(p.hauteur, 1)} mm — ${fp}`;
}

/**
 * Un relevé fait sur le gabarit de l'imprimeur.
 *
 * Vide au départ, jamais prérempli : un chiffre par défaut se lirait comme une mesure,
 * et une planche composée sur un dos inventé ne se voit qu'au massicot.
 */
function champReleve(id, libelle, valeur) {
  const l = h('label', undefined, 'petit');
  const i = h('input');
  i.type = 'number';
  i.id = id;
  i.min = 0;
  i.step = 0.1;
  i.value = valeur === null || valeur === undefined ? '' : String(valeur);
  // Aucun écouteur : le relevé part avec le reste du formulaire, au clic sur le verbe.
  // Il n'y a plus de commande d'écriture directe à qui l'envoyer.
  l.append(h('span', libelle), i);
  return l;
}

/**
 * Les fichiers d'un package : leur répertoire une fois, leurs noms ensuite.
 *
 * Un package écrit tous ses fichiers au même endroit, et redire soixante-dix caractères
 * de chemin identiques à chaque ligne coûtait deux lignes de plus par livrable —
 * l'ascenseur de la Livraison se payait en redites. Coupés ainsi, les noms tiennent sur
 * une ligne au lieu de se replier au milieu d'un mot.
 *
 * Si les fichiers ne partagent pas leur répertoire, chacun reprend son chemin entier :
 * un chemin long se lit, un chemin faux se suit jusqu'à un fichier qui n'y est pas. Les
 * deux séparateurs sont reconnus — l'application est aussi empaquetée pour Windows, et
 * un `\` pris pour une lettre rendrait le groupement muet là-bas.
 */
function cheminsGroupes(chemins) {
  const dossier = (c) => c.slice(0, Math.max(c.lastIndexOf('/'), c.lastIndexOf('\\')) + 1);
  const commun = chemins.length ? dossier(chemins[0]) : '';
  if (!commun || !chemins.every((c) => dossier(c) === commun)) return chemins;
  return [commun, chemins.map((c) => c.slice(commun.length)).join('   ')];
}

async function packager() {
  const combien = projet.livraison.livrables.length;
  // Générer compose : le projet revient mesuré, et le pied le relit là où il est
  // enregistré — sans quoi il dirait « dos non composé » sous une ligne qui vient de
  // donner le dos. Les comptes rendus entrent dans les lignes, il n'y a plus de zone
  // intermédiaire où les lire.
  await pendantQueCaCompose(
    $('btToutRegenerer'),
    `composition de ${combien} package(s)…`,
    () => invoke('packager')
  );
}

/** Une taille de fichier, en unités qu'on lit d'un coup d'œil. */
function poids(octets) {
  return octets >= 1024 * 1024
    ? `${nb(octets / (1024 * 1024), 1)} Mo`
    : `${Math.round(octets / 1024)} Ko`;
}

/**
 * Le compte rendu des ebooks : les deux chemins, leur poids, et ce qui s'est passé de
 * travers sans faire échouer la génération.
 *
 * La police non embarquée n'est pas une erreur : le livre reste juste, seul son œil
 * change. Elle se lit donc dans le compte rendu, à côté des chemins, et non en rouge à
 * la place d'un résultat qui existe.
 */
function afficherEbooks(r) {
  const box = $('ebooks');
  box.replaceChildren();
  for (const [chemin, octets] of [[r.pdf, r.octets_pdf], [r.epub, r.octets_epub]]) {
    box.append(h('p', `${chemin}   (${poids(octets)})`, 'chemin'));
  }
  if (r.police_non_embarquee) {
    box.append(h('p', `Police « ${r.police_non_embarquee} » introuvable : l'EPUB est `
      + `dans l'écriture du lecteur. Le texte, lui, est celui du livre.`, 'note'));
  }
  // Celle-ci, en revanche, touche le PDF : c'est le fichier qu'on lira, et il ne suit
  // pas la maquette.
  if (r.polices_introuvables.length) {
    box.append(h('p', 'Police introuvable, composé dans une écriture de repli : '
      + `${r.polices_introuvables.join(', ')}. Le PDF ne suit pas la maquette.`,
    'note alerte'));
  }
  box.hidden = false;
}

async function ebooks() {
  const bt = $('btEbooks');
  bt.disabled = true;
  $('ebooks').hidden = true;
  $('etatEbooks').className = 'etat';
  $('etatEbooks').textContent = 'composition du PDF et de l’EPUB…';
  try {
    afficherEbooks(await invoke('ebook_generer'));
    $('etatEbooks').textContent = '';
  } catch (e) {
    $('etatEbooks').textContent = String(e);
    $('etatEbooks').className = 'etat erreur';
  } finally {
    bt.disabled = false;
  }
}

