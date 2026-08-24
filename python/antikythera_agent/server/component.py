"""Static serving + manifest untuk bundle jco (unit U22, D4).

Sumber kebenaran:
- contracts/shared/wire_protocol.golden.json  (entry `component_manifest`)
- documentation/WIRE_PROTOCOL.md  §2.6
- documentation/DECISIONS_RUNTIME_BRIDGE.md  D1/D4

Kontrak sambungan ke transport (U31):
- `GET /antikythera/v1/component/manifest` dijawab dari `manifest()` —
  JSON persis shape golden, tanpa kunci tambahan.
- `GET /antikythera/v1/component/{path}` dijawab dari `resolve(path)` —
  transport WAJIB meng-URL-decode `{path}` SEBELUM memanggil resolve;
  percent-encoding bukan urusan unit ini.
- `is_known_entry(entry)` memverifikasi entry manifest tersedia di bundle.

Amplop fail-fast (desain U1): bundle adalah artefak build, bukan source —
`manifest()` dan `resolve(ENTRY)` menolak bundle yang belum dibangun
(`RuntimeError`, remedy: task build-python-wasm). Path non-entry pada
resolve() dan is_known_entry() tetap lenient (None/False) — pemetaan 404
tetap urusan transport. Constructor tetap lazy (tanpa I/O).

Batas validitas: file dilayani as-is (dibaca penuh per permintaan, tanpa
cache atau modifikasi) — sesuai amplop D3 (tens of concurrent clients).
"""

from __future__ import annotations

from pathlib import Path
from typing import Dict, Optional, Tuple

from antikythera_agent.utils import get_version

#: URL directory bundle di wire protocol (nilai `base` manifest, D4).
BASE_PATH = "/antikythera/v1/component/"

#: Route endpoint manifest; U31 memasang handler pada path ini.
MANIFEST_PATH = "/antikythera/v1/component/manifest"

#: Nama file entry ESM di dalam bundle (D4; sama dengan output jco npm).
ENTRY = "antikythera-sdk.js"

#: Pasangan MIME yang disahkan D4/WIRE_PROTOCOL §2.6 — satu-satunya yang dikontrak.
MIME_TYPES = {
    ".js": "text/javascript",
    ".wasm": "application/wasm",
}

#: Default binary RFC 2046 untuk ekstensi yang tidak terdaftar.
FALLBACK_MIME = "application/octet-stream"


class ComponentServer:
    """Menyajikan bundle jco (entry + file pendukung) dan manifestnya.

    Constructor tidak melakukan I/O (lazy): bundle_dir boleh belum ada di
    disk. Integritas bundle diverifikasi per permintaan oleh
    `_ensure_entry_available` — manifest() dan resolve(ENTRY) fail-fast;
    resolve path lain dan is_known_entry() tetap melaporkan ketiadaan
    sebagai None/False.
    """

    def __init__(self, bundle_dir: Optional[Path] = None) -> None:
        if bundle_dir is None:
            bundle_dir = Path(__file__).resolve().parents[1] / "component"
        self.bundle_dir = Path(bundle_dir).resolve()

    def _ensure_entry_available(self) -> None:
        """Gate integritas bundle (U1): entry adalah artefak identitas bundle.

        Dua tepi yang ditolak: bundle_dir tidak ada di disk, atau dir ada
        tetapi ENTRY tidak di dalamnya — keduanya berarti bundle belum
        dibangun; remedy tunggalnya task build-python-wasm.
        """
        if self.bundle_dir.is_dir() and (self.bundle_dir / ENTRY).is_file():
            return
        raise RuntimeError(
            f"Bundle komponen tidak tersedia: {self.bundle_dir} tidak memuat "
            f"entry {ENTRY!r}. Jalankan task build-python-wasm untuk "
            f"membangun bundle terlebih dahulu."
        )

    def manifest(self) -> Dict[str, str]:
        """Manifest bundle — persis shape golden `component_manifest`.

        Raises:
            RuntimeError: bila bundle belum dibangun (lihat amplop U1).
        """
        self._ensure_entry_available()
        return {
            "base": BASE_PATH,
            "entry": ENTRY,
            "version": get_version(),
        }

    def resolve(self, path: str) -> Optional[Tuple[bytes, str]]:
        """Konten + MIME file di dalam bundle, atau None bila tidak ada.

        Menerima path relatif posix-style yang SUDAH di-decode transport.
        Upaya traversal (keluar dari bundle_dir) dikembalikan sebagai None.
        Permintaan terhadap ENTRY dievaluasi dulu lewat gate integritas
        (fail-fast U1); path lain mempertahankan perilaku lenient None agar
        traversal guard dan pemetaan 404 transport tidak berubah.
        """
        if path == ENTRY:
            self._ensure_entry_available()
        candidate = self._locate(path)
        if candidate is None:
            return None
        return candidate.read_bytes(), MIME_TYPES.get(candidate.suffix, FALLBACK_MIME)

    def is_known_entry(self, entry: str) -> bool:
        """True bila `entry` menunjuk file nyata di dalam bundle."""
        return self._locate(entry) is not None

    def _locate(self, path: str) -> Optional[Path]:
        """Resolusi path ke file nyata di dalam bundle_dir, atau None.

        Dua lapis traversal guard: (1) tolak eksplisit separator `..`,
        komponen kosong/`.`, path absolut, dan backslash (separator Windows
        yang tidak pernah sah di URL bundle); (2) verifikasi kontainmen
        hasil resolve() terhadap root — menutup symlink dan keanehan lain
        yang lolos lapis pertama. `pathlib` mengganti base bila di-join
        dengan path absolut (terverifikasi), jadi lapis (2) wajib ada.
        """
        if not path or path.startswith(("/", "\\")) or "\\" in path:
            return None
        parts = path.split("/")
        if any(part in ("", ".", "..") for part in parts):
            return None
        candidate = (self.bundle_dir / path).resolve()
        if not candidate.is_relative_to(self.bundle_dir):
            return None
        if not candidate.is_file():
            return None
        return candidate
