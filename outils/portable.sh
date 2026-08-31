#!/usr/bin/env bash
# Assemble l'archive portable pour Windows.
#
# Tauri 2 n'offre sous Windows que les cibles `nsis` et `msi` : il n'y a pas de cible
# « portable » à demander à `tauri build`, l'archive se monte à la main. Ici et non dans
# le workflow, pour la même raison que `typst.sh` et `polices.sh` : ce qui n'existe que
# dans la CI ne se vérifie que dans la CI, et personne n'ouvrirait le portable avant le
# premier utilisateur.
set -euo pipefail

ICI="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TRIPLE="x86_64-pc-windows-msvc"
EXE="$ICI/src-tauri/target/release/ozalid-studio.exe"
SIDECAR="$ICI/src-tauri/binaries/typst-${TRIPLE}.exe"
POLICES="$ICI/src-tauri/fonts"
SORTIE="$ICI/src-tauri/target/portable"

for f in "$EXE" "$SIDECAR"; do
  [ -f "$f" ] || { echo "absent : $f" >&2; exit 1; }
done
ls "$POLICES"/*.ttf >/dev/null 2>&1 || { echo "aucune police dans $POLICES" >&2; exit 1; }

# La même source de vérité que le contrôle de tag du job `publier` : la version de
# l'application est celle de tauri.conf.json, et rien d'autre ne la porte.
VERSION="$(node -p "require('$ICI/src-tauri/tauri.conf.json').version")"
NOM="Ozalid Studio $VERSION"

rm -rf "$SORTIE"
mkdir -p "$SORTIE/$NOM"
cp "$EXE" "$SORTIE/$NOM/ozalid-studio.exe"
# Renommé, comme le bundle Tauri le fait d'un `externalBin` : `commands.rs::binaire_typst`
# cherche « typst.exe » à côté de l'exécutable, pas le nom triplé.
cp "$SIDECAR" "$SORTIE/$NOM/typst.exe"
mkdir -p "$SORTIE/$NOM/fonts"
cp "$POLICES"/* "$SORTIE/$NOM/fonts/"
# Le marqueur : vide, nommé d'après l'exécutable. C'est lui, et lui seul, qui fait
# écrire les réglages dans le « donnees/ » voisin plutôt que sur la machine hôte.
# `donnees/` n'est pas livré : il naît au premier lancement, et une archive qui le
# contiendrait ferait croire à un dossier qu'on peut écraser en mettant à jour.
: > "$SORTIE/$NOM/ozalid-studio.portable"

ARCHIVE="$SORTIE/ozalid-studio_${VERSION}_x64-portable.zip"
# `tar` et non `zip` : `zip` n'est pas garanti dans le Git Bash des runners Windows,
# où ce script tourne — la même raison qui fait passer `typst.sh` par `tar` pour
# l'extraction. Le `-a` prend le format sur l'extension, et deflate.
(cd "$SORTIE" && tar -a -cf "$ARCHIVE" "$NOM")

echo "$ARCHIVE"
