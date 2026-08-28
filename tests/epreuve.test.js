'use strict';

// Câblage de l'intérieur et de l'épreuve : la police du projet qui paraît au panneau,
// le réglage qui redescend jusqu'au Rust, et l'épreuve qui se tire sans rien composer
// au préalable. Le rendu de l'épreuve se vérifie en la composant, pas ici.

const test = require('node:test');
const assert = require('node:assert');
const { charge } = require('./dom_shim');

const LULU = {
  cle: 'lulu-108x175-broche', pod: 'lulu', format: '108x175', reliure: 'broche',
  libelle: 'Lulu — poche 108 × 175',
  largeur: 108, hauteur: 175, fond_perdu: 3.175,
};

const PROJET = {
  chemin: '/livres/LHC.ozalid',
  livre: {
    titre: 'Les Heures creuses', titre_page: null, auteur: 'Ivan Pjig',
    genre: 'roman', copyright: '', chapitres: 64,
  },
  manuscrit_absent: false,
  modifie: false,
  manuscrit_source: '/x/WIP7.md',
  chapitres_trouves: 64,
  mots: 49344,
  couverture: null,
  couverture_importee: false,
  images: ['couverture.jpg'],
  // Les onze tailles telles que le Rust les sert : `Interieur` n'en tait aucune, et
  // une fixture qui en oublierait ferait paraître des champs vides là où l'application
  // en montre toujours onze.
  interieur: {
    police: 'Alegreya',
    corps: 9.5, faux_titre: 11, page_titre_auteur: 10.5, page_titre_titre: 15,
    page_titre_genre: 10, copyright: 8, dedicace: 9.5, numero: 13,
    titre_section: 10, ouverture_piece: 10, folio: 8,
  },
  envois: { main: { mode: 'police', police: 'Caveat' }, liste: [] },
  livraison: {
    livrables: [{
      cle: 'lulu-108x175-broche-standard', gabarit: 'lulu-108x175-broche',
      pod: 'lulu', format: '108x175', reliure: 'broche', papier: 'standard',
      finition: null, dos_mm: null, fond_perdu_mm: null, compose: null,
    }],
    courant: 'lulu-108x175-broche-standard',
  },
};

/** Fausse implémentation des commandes Rust. `sur` surcharge une commande. */
function faux(providers, sur = {}) {
  return async (cmd, args) => {
    if (cmd === 'providers_liste') return providers;
    if (cmd === 'pods_liste') return [{
      cle: 'lulu', nom: 'Lulu',
      formats: [{ cle: '108x175', nom: 'poche 108 × 175' }],
      reliures: [{ cle: 'broche', nom: 'Broché — dos carré collé', non_outille: null }],
      finitions: [],
      papiers: [{ cle: 'standard', libelle: 'Papier standard', teinte: '#ffffff', dos_publie: true }],
    }];
    if (cmd === 'catalogue_refus') return [];
    if (cmd === 'polices_liste') return ['Bodoni Moda', 'Archivo', 'Spectral'];
    if (cmd === 'polices_texte_liste') return ['EB Garamond', 'Alegreya', 'Cardo'];
    if (cmd === 'jetons_liste') return ['%TITRE%', '%AUTEUR%', '%GENRE%', '%EDITEUR%', '%COLLECTION%', '%MONOGRAMME%'];
    if (cmd === 'mains_liste') return ['Caveat', 'Dancing Script'];
    if (cmd === 'maquettes_liste') return [{ cle: 'bandeau', libelle: 'Bandeau' }];
    if (cmd === 'couverture_apercu') return { image: 'data:image/png;base64,AAAA', reperes: null };
    if (cmd in sur) {
      const v = sur[cmd];
      return typeof v === 'function' ? v(args) : v;
    }
    // Le démarrage et la garde envoient ces trois commandes sans qu'aucun test ne les
    // demande : sans réponse ici, elles lèveraient avant que rien ne soit vérifié.
    if (cmd === 'recents_liste') return [];
    if (cmd === 'garde_modifications') return 'ignorer';
    if (cmd === 'interface_prete') return null;
    // Importer un `livre.toml`, c'est apporter un manuscrit — donc composer, sans
    // qu'aucun test de ce fichier ne le demande. Une commande inattendue leur
    // remonterait une erreur dans l'entête.
    //
    // La mesure **doit** entrer dans le projet rendu, comme le Rust le fait : la veille
    // relance tant que le livrable visé n'en porte pas, et une composition qui
    // réussirait sans rien déposer tournerait en boucle.
    if (cmd === 'composer') {
      const mesure = { pages: 262, gouttiere: 25, blanche: true, dos: 16.513 };
      return {
        ...mesure,
        chapitres: 64,
        pdf: '/livres/LHC/lulu/interieur-lulu.pdf',
        polices_introuvables: [],
        projet: {
          ...PROJET,
          livraison: {
            ...PROJET.livraison,
            deja_compose: true,
            livrables: PROJET.livraison.livrables.map((d) => (
              d.cle === PROJET.livraison.courant ? { ...d, compose: mesure } : d
            )),
          },
        },
      };
    }
    // L'accès au modèle de diffusion se lit au démarrage : il appartient à la
    // machine, et l'écran le montre avant qu'aucun projet ne soit ouvert.
    if (cmd === 'diffusion_lire') return { url: '', modele: '', cle_posee: false };
    throw new Error(`commande inattendue : ${cmd}`);
  };
}

/* ---------- intérieur ---------- */

test('la police d\'intérieur du projet est celle qui paraît au panneau', async () => {
  const { els } = await charge({
    invoke: faux([LULU], { projet_importer: PROJET }),
    open: async () => '/dev/ozalid/build/LHC/livre.toml',
  });
  await els.get('btImporter').declenche('click');
  // L'ouverture arrive au Livre, et la police y est : elle a quitté l'étape Intérieur
  // pour le bloc qui ferme celle-ci, sans plus rien à traverser pour l'atteindre.
  assert.strictEqual(els.get('etapeLivre').hidden, false);
  assert.strictEqual(els.get('inPoliceInterieur').value, 'Alegreya');
});

/**
 * Le réglage doit atteindre le Rust : un sélecteur qui change d'apparence sans rien
 * enregistrer laisserait composer dans une autre police que celle qu'on voit.
 */
test('changer la police enregistre le réglage dans le projet', async () => {
  let recu = null;
  const { els } = await charge({
    invoke: faux([LULU], {
      projet_importer: PROJET,
      interieur_modifier: (args) => {
        recu = args.interieur;
        return { ...PROJET, interieur: args.interieur };
      },
    }),
    open: async () => '/dev/ozalid/build/LHC/livre.toml',
  });
  await els.get('btImporter').declenche('click');
  els.get('inPoliceInterieur').value = 'Cardo';
  await els.get('inPoliceInterieur').declenche('change');
  // Recopié : l'objet vient du contexte où s'exécute app.js, donc d'un autre realm.
  // Les tailles voyagent avec, inchangées : le formulaire commet l'intérieur entier à
  // chaque geste, et en omettre une la ramènerait à son défaut sans le dire.
  assert.deepStrictEqual({ ...recu }, { ...PROJET.interieur, police: 'Cardo' });
  assert.strictEqual(els.get('inPoliceInterieur').value, 'Cardo');
});

/**
 * Une taille saisie doit atteindre le Rust, et elle seule : c'est la pagination
 * qu'elle déplace, donc l'épaisseur du dos. Un champ qui changerait d'apparence sans
 * rien enregistrer laisserait composer dans le corps qu'on ne voit plus.
 *
 * Le champ est lu en nombre et non en chaîne — `Interieur` attend des `f64`, et serde
 * refuse `"10.5"`. Le test le vérifie par le type, pas seulement par la valeur.
 */
test('changer une taille enregistre le réglage dans le projet', async () => {
  let recu = null;
  const { els } = await charge({
    invoke: faux([LULU], {
      projet_importer: PROJET,
      interieur_modifier: (args) => {
        recu = args.interieur;
        return { ...PROJET, interieur: args.interieur };
      },
    }),
    open: async () => '/dev/ozalid/build/LHC/livre.toml',
  });
  await els.get('btImporter').declenche('click');
  els.get('inTailleTitre').value = '18.5';
  await els.get('inTailleTitre').declenche('change');
  assert.deepStrictEqual(
    { ...recu },
    { ...PROJET.interieur, page_titre_titre: 18.5 },
    'la taille saisie part seule, les dix autres inchangées',
  );
  assert.strictEqual(typeof recu.page_titre_titre, 'number');
  assert.strictEqual(els.get('inTailleTitre').value, '18.5');
});

/**
 * Une police refusée par le Rust doit rester lisible à l'écran, et le panneau doit
 * revenir à ce que le projet porte vraiment : laisser le sélecteur sur un choix non
 * enregistré ferait croire qu'on compose dans une police qui n'est pas celle-là.
 */
test('une police refusée est dite, et le panneau revient au projet', async () => {
  const { els } = await charge({
    invoke: faux([LULU], {
      projet_importer: PROJET,
      interieur_modifier: () => {
        throw 'police d\'intérieur inconnue : « Oswald ».';
      },
    }),
    open: async () => '/dev/ozalid/build/LHC/livre.toml',
  });
  await els.get('btImporter').declenche('click');
  els.get('inPoliceInterieur').value = 'Cardo';
  await els.get('inPoliceInterieur').declenche('change');
  // Le refus passe par `tente()`, qui écrit dans l'entête et non dans l'étape.
  assert.match(els.get('alerte').textContent, /police d'intérieur inconnue/);
  assert.strictEqual(els.get('alerte').className, 'etat erreur');
  assert.strictEqual(
    els.get('inPoliceInterieur').value,
    'Alegreya',
    'le panneau reste sur une police que le projet ne porte pas'
  );
});

/* ---------- épreuve ---------- */

/**
 * L'épreuve ne dépend d'aucune pagination ni d'aucun imprimeur : elle doit pouvoir
 * être tirée dès qu'un manuscrit est là, sans intérieur composé au préalable.
 */
test('l\'épreuve se tire sans intérieur composé', async () => {
  let corps = null;
  const { els } = await charge({
    invoke: faux([LULU], {
      projet_importer: PROJET,
      epreuve_tirer: (args) => {
        corps = args.corpsPt;
        return '/livres/LHC/epreuve.pdf';
      },
    }),
    open: async () => '/dev/ozalid/build/LHC/livre.toml',
  });
  await els.get('btImporter').declenche('click');
  await els.get('btEpreuve').declenche('click');
  assert.strictEqual(corps, 12);
  assert.strictEqual(els.get('cheminEpreuve').textContent, '/livres/LHC/epreuve.pdf');
  assert.strictEqual(els.get('etatEpreuve').textContent, '');
});

/**
 * Un échec doit être dit et ne pas laisser en place le chemin d'une épreuve précédente :
 * on irait relire un PDF périmé en croyant relire celui qu'on vient de demander.
 */
test('une épreuve en échec est signalée et efface le chemin précédent', async () => {
  let echoue = false;
  const { els } = await charge({
    invoke: faux([LULU], {
      projet_importer: PROJET,
      epreuve_tirer: () => {
        if (echoue) throw 'enregistrer le projet avant de composer.';
        return '/livres/LHC/epreuve.pdf';
      },
    }),
    open: async () => '/dev/ozalid/build/LHC/livre.toml',
  });
  await els.get('btImporter').declenche('click');
  await els.get('btEpreuve').declenche('click');
  assert.strictEqual(els.get('cheminEpreuve').textContent, '/livres/LHC/epreuve.pdf');

  echoue = true;
  await els.get('btEpreuve').declenche('click');
  assert.match(els.get('etatEpreuve').textContent, /enregistrer le projet/);
  assert.strictEqual(els.get('etatEpreuve').className, 'etat erreur');
  assert.strictEqual(els.get('cheminEpreuve').textContent, '', 'chemin périmé laissé à l\'écran');
  assert.strictEqual(els.get('btEpreuve').disabled, false, 'bouton laissé bloqué');
});
