//! Ce qui dit qu'un package n'est plus celui du projet qu'on a sous les yeux.
//!
//! Deux empreintes par livrable — l'intérieur, la couverture — parce qu'elles répondent à
//! deux questions différentes : laquelle des deux moitiés a bougé, et un intérieur déjà
//! composé peut-il resservir alors que la couverture, elle, a changé.

/// Un condensé FNV-1a 64 bits, en seize caractères hexadécimaux.
///
/// **Écrit ici et non repris de `commands::empreinte`**, qui repose sur `DefaultHasher`.
/// Celle-là nomme un répertoire de rendus : une valeur qui change fabrique un répertoire
/// neuf et l'on recalcule, personne ne le voit. Celle-ci est écrite dans le `.ozalid` et
/// relue par un binaire recompilé — or la bibliothèque standard ne garantit pas que
/// `DefaultHasher` rende la même valeur d'une version de Rust à l'autre. Une mise à jour de
/// l'application marquerait alors tous les packages périmés d'un coup, sans que rien ne
/// l'explique. FNV-1a, lui, est une spécification : il ne bougera jamais.
pub fn condense(octets: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for o in octets {
        h ^= u64::from(*o);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les valeurs sont **gelées**, et c'est tout l'intérêt du test : cette empreinte est
    /// écrite dans le `.ozalid` et relue par un binaire qu'on aura recompilé entre-temps.
    /// Un algorithme qu'on changerait sans y penser marquerait d'un coup tous les packages
    /// de tous les projets comme périmés, sans que rien à l'écran puisse l'expliquer. Les
    /// trois vecteurs sont ceux de la spécification FNV-1a 64 bits.
    #[test]
    fn le_condense_est_gele() {
        assert_eq!(condense(b""), "cbf29ce484222325");
        assert_eq!(condense(b"a"), "af63dc4c8601ec8c");
        assert_eq!(condense(b"ozalid"), "dc0fb47ed8d84474");
    }

    /// Deux entrées voisines ne se confondent pas : sans quoi changer une lettre du titre
    /// laisserait la couverture marquée à jour.
    #[test]
    fn deux_entrees_voisines_ne_se_condensent_pas_pareil() {
        assert_ne!(condense(b"a"), condense(b"b"));
    }
}
