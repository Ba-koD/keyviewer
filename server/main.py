import asyncio
import json
import threading
import time
from dataclasses import dataclass, asdict, field
from pathlib import Path
from typing import List, Optional, Set, Dict

import psutil
import keyboard  # Requires admin privileges on Windows for global hook
import uvicorn
from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.responses import HTMLResponse, RedirectResponse, JSONResponse
from fastapi.middleware.cors import CORSMiddleware
from starlette.staticfiles import StaticFiles
import sys
import os

# Ensure this sentinel exists regardless of import success
except_import_error: Optional[Exception] = None

try:
	import win32gui
	import win32process
	except_import_error = None
except Exception as e:
	except_import_error = e

try:
	import win32con
except Exception:
	win32con = None


# -------------------------------
# App config
# -------------------------------
@dataclass
class OverlayConfig:
	fade_in_ms: int = 120
	fade_out_ms: int = 120
	chip_bg: str = "rgba(0,0,0,0.6)"
	chip_fg: str = "#ffffff"
	chip_gap: int = 8
	chip_pad_v: int = 10
	chip_pad_h: int = 14
	chip_radius: int = 10
	chip_font_px: int = 24
	chip_font_weight: int = 700
	background: str = "rgba(0,0,0,0.0)"  # page background for OBS
	cols: int = 8
	rows: int = 1
	single_line: bool = False
	single_line_scale: int = 90  # percent, only applied when single_line
	align: str = "center"  # left|center|right
	direction: str = "ltr"  # ltr|rtl


@dataclass
class AppConfig:
	port: int = 8000
	overlay: OverlayConfig = field(default_factory=OverlayConfig)


def _config_dir() -> Path:
	appdata = os.getenv("APPDATA")
	if not appdata:
		return Path.home() / ".keyqueueviewer"
	return Path(appdata) / "KeyQueueViewer"


def _config_path() -> Path:
	cfg_dir = _config_dir()
	cfg_dir.mkdir(parents=True, exist_ok=True)
	return cfg_dir / "config.json"


def load_app_config() -> AppConfig:
	p = _config_path()
	if p.exists():
		try:
			data = json.loads(p.read_text("utf-8"))
			port = int(data.get("port", 8000))
			ov = data.get("overlay", {}) or {}
			overlay = OverlayConfig(
				fade_in_ms=int(ov.get("fade_in_ms", 120)),
				fade_out_ms=int(ov.get("fade_out_ms", 120)),
				chip_bg=str(ov.get("chip_bg", "rgba(0,0,0,0.6)")),
				chip_fg=str(ov.get("chip_fg", "#ffffff")),
				chip_gap=int(ov.get("chip_gap", 8)),
				chip_pad_v=int(ov.get("chip_pad_v", 10)),
				chip_pad_h=int(ov.get("chip_pad_h", 14)),
				chip_radius=int(ov.get("chip_radius", 10)),
				chip_font_px=int(ov.get("chip_font_px", 24)),
				chip_font_weight=int(ov.get("chip_font_weight", 700)),
				background=str(ov.get("background", "rgba(0,0,0,0.0)")),
				cols=int(ov.get("cols", 8)),
				rows=int(ov.get("rows", 1)),
				single_line=bool(ov.get("single_line", False)),
				single_line_scale=int(ov.get("single_line_scale", 90)),
				align=str(ov.get("align", "center")),
				direction=str(ov.get("direction", "ltr")),
			)
			return AppConfig(port=port, overlay=overlay)
		except Exception:
			return AppConfig()
	return AppConfig()


def save_app_config(cfg: AppConfig) -> None:
	p = _config_path()
	data = {"port": cfg.port, "overlay": asdict(cfg.overlay)}
	p.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")


# -------------------------------
# Key state management
# -------------------------------
class KeyQueueState:
	def __init__(self) -> None:
		# Track by physical key id (scan code if available)
		self._pressed_order: List[object] = []  # key_id list to preserve order
		self._pressed_set: Set[object] = set()
		self._label_by_id: Dict[object, str] = {}
		self._lock = threading.Lock()

	@staticmethod
	def _normalize_key_name(raw_name: str) -> str:
		name = raw_name or ""
		name = name.replace("_", " ")
		if len(name) == 1:
			# Keep as-is for symbols (e.g., '!'), uppercase for letters
			return name.upper() if name.isalpha() else name
		# Common aliases
		aliases: Dict[str, str] = {
			"space": "SPACE",
			"enter": "ENTER",
			"return": "ENTER",
			"tab": "TAB",
			"ctrl": "CTRL",
			"control": "CTRL",
			"alt": "ALT",
			"shift": "SHIFT",
			"caps lock": "CAPS",
			"backspace": "BKSP",
			"esc": "ESC",
			"escape": "ESC",
			"left": "LEFT",
			"right": "RIGHT",
			"up": "UP",
			"down": "DOWN",
		}
		lower = name.lower()
		if lower in aliases:
			return aliases[lower]
		return name.upper()

	@staticmethod
	def _key_id(scan_code: Optional[int], name: Optional[str]) -> object:
		if scan_code is not None:
			return int(scan_code)
		return f"NAME:{name or ''}"

	def clear_all(self) -> bool:
		with self._lock:
			if not self._pressed_order:
				return False
			self._pressed_order.clear()
			self._pressed_set.clear()
			self._label_by_id.clear()
			return True

	def on_key_down(self, scan_code: Optional[int], key_name: str) -> bool:
		key_id = self._key_id(scan_code, key_name)
		label = self._normalize_key_name(key_name)
		with self._lock:
			if key_id in self._pressed_set:
				return False
			self._pressed_set.add(key_id)
			self._pressed_order.append(key_id)
			self._label_by_id[key_id] = label
			return True

	def on_key_up(self, scan_code: Optional[int], key_name: str) -> bool:
		key_id = self._key_id(scan_code, key_name)
		with self._lock:
			if key_id not in self._pressed_set:
				return False
			self._pressed_set.remove(key_id)
			try:
				self._pressed_order.remove(key_id)
			except ValueError:
				pass
			self._label_by_id.pop(key_id, None)
			return True

	def snapshot(self) -> List[str]:
		with self._lock:
			return [self._label_by_id.get(key_id, "?") for key_id in self._pressed_order]


# -------------------------------
# Target window matching
# -------------------------------
@dataclass
class TargetConfig:
	mode: str = "disabled"  # "disabled" | "title" | "process" | "hwnd" | "class" | "all"
	value: Optional[str] = None


def _resource_dir() -> Path:
	# When frozen by PyInstaller, sys._MEIPASS contains temp extraction dir
	if getattr(sys, 'frozen', False) and hasattr(sys, '_MEIPASS'):
		return Path(sys._MEIPASS)  # type: ignore
	return Path(__file__).resolve().parent.parent


def _get_foreground_info() -> Dict[str, Optional[str]]:
	if except_import_error is not None:
		return {"hwnd": None, "title": None, "pid": None, "process_name": None, "class": None}
	try:
		hwnd = win32gui.GetForegroundWindow()
		title = win32gui.GetWindowText(hwnd) or ""
		pid = win32process.GetWindowThreadProcessId(hwnd)[1]
		pname = None
		wclass = None
		try:
			pname = psutil.Process(pid).name()
		except Exception:
			pname = None
		try:
			wclass = win32gui.GetClassName(hwnd)
		except Exception:
			wclass = None
		return {"hwnd": str(hwnd), "title": title, "pid": str(pid), "process_name": pname, "class": wclass}
	except Exception:
		return {"hwnd": None, "title": None, "pid": None, "process_name": None, "class": None}


def _matches(fg: Dict[str, Optional[str]], target: TargetConfig) -> bool:
	if target.mode == "disabled":
		return False
	if target.mode == "all":
		# Accept any foreground window
		return True if fg.get("hwnd") else False
	if not fg.get("hwnd"):
		return False
	if target.mode == "hwnd":
		return str(fg.get("hwnd")) == str(target.value)
	if target.mode == "title":
		title = fg.get("title") or ""
		return (target.value or "").lower() in title.lower()
	if target.mode == "process":
		pname = (fg.get("process_name") or "").lower()
		return pname == (str(target.value or "").lower())
	if target.mode == "class":
		wclass = (fg.get("class") or "").lower()
		return wclass == (str(target.value or "").lower())
	return False


# -------------------------------
# WebSocket connection manager
# -------------------------------
class ConnectionManager:
	def __init__(self) -> None:
		self.active: Set[WebSocket] = set()
		self._lock = asyncio.Lock()

	async def connect(self, websocket: WebSocket) -> None:
		await websocket.accept()
		async with self._lock:
			self.active.add(websocket)

	async def disconnect(self, websocket: WebSocket) -> None:
		async with self._lock:
			if websocket in self.active:
				self.active.remove(websocket)

	async def broadcast(self, message: dict) -> None:
		payload = json.dumps(message)
		async with self._lock:
			if not self.active:
				return
			to_remove: List[WebSocket] = []
			for ws in list(self.active):
				try:
					await ws.send_text(payload)
				except Exception:
					to_remove.append(ws)
			for ws in to_remove:
				try:
					self.active.remove(ws)
				except KeyError:
					pass


# -------------------------------
# App setup
# -------------------------------
app = FastAPI()
app.add_middleware(
	CORSMiddleware,
	allow_origins=["*"],
	allow_credentials=True,
	allow_methods=["*"],
	allow_headers=["*"],
)

BASE_DIR = _resource_dir()
WEB_DIR = BASE_DIR / "web"
WEB_DIR.mkdir(parents=True, exist_ok=True)

app.mount("/static", StaticFiles(directory=str(WEB_DIR)), name="static")

state = KeyQueueState()
manager = ConnectionManager()
target_config = TargetConfig()
app_config = load_app_config()

event_loop: Optional[asyncio.AbstractEventLoop] = None


# -------------------------------
# Keyboard hook thread
# -------------------------------
class KeyboardHookThread(threading.Thread):
	def __init__(self) -> None:
		super().__init__(daemon=True)
		self._running = True

	def stop(self) -> None:
		self._running = False

	def _post_update(self) -> None:
		if event_loop is None:
			return
		keys = state.snapshot()
		try:
			asyncio.run_coroutine_threadsafe(
				manager.broadcast({"type": "keys", "keys": keys}), event_loop
			)
		except Exception:
			pass

	def _handle_event(self, event: keyboard.KeyboardEvent) -> None:
		# Only act if target window is focused
		fg = _get_foreground_info()
		if not _matches(fg, target_config):
			return
		changed = False
		if event.event_type == "down":
			changed = state.on_key_down(event.scan_code, event.name)
		elif event.event_type == "up":
			changed = state.on_key_up(event.scan_code, event.name)
		if changed:
			self._post_update()

	def run(self) -> None:
		keyboard.hook(self._handle_event, suppress=False)
		# Block to keep the thread alive
		keyboard.wait()


# -------------------------------
# Foreground focus watcher thread
# -------------------------------
class FocusWatcherThread(threading.Thread):
	def __init__(self) -> None:
		super().__init__(daemon=True)
		self._running = True

	def stop(self) -> None:
		self._running = False

	def run(self) -> None:
		while self._running:
			try:
				fg = _get_foreground_info()
				if not _matches(fg, target_config):
					# Clear any held keys when target loses focus
					if state.clear_all() and event_loop is not None:
						asyncio.run_coroutine_threadsafe(
							manager.broadcast({"keys": []}), event_loop
						)
			except Exception:
				pass
			time.sleep(0.10)


kb_thread = KeyboardHookThread()
focus_thread = FocusWatcherThread()


# -------------------------------
# Routes
# -------------------------------
@app.get("/")
async def root_redirect() -> RedirectResponse:
	return RedirectResponse(url="/control", status_code=302)


@app.get("/overlay")
async def get_overlay() -> HTMLResponse:
	index_html = (WEB_DIR / "index.html").read_text(encoding="utf-8") if (WEB_DIR / "index.html").exists() else "<html><body>Overlay not found</body></html>"
	return HTMLResponse(index_html)


@app.get("/control")
async def get_control() -> HTMLResponse:
	ctl_html = (WEB_DIR / "control.html").read_text(encoding="utf-8") if (WEB_DIR / "control.html").exists() else "<html><body>Control not found</body></html>"
	return HTMLResponse(ctl_html)


@app.websocket("/ws")
async def websocket_endpoint(ws: WebSocket) -> None:
	await manager.connect(ws)
	try:
		# Send initial state
		await ws.send_text(json.dumps({"keys": state.snapshot()}))
		while True:
			# Keep connection alive; we do not expect client messages
			await ws.receive_text()
	except WebSocketDisconnect:
		await manager.disconnect(ws)
	except Exception:
		await manager.disconnect(ws)


@app.get("/api/windows")
async def api_windows() -> List[Dict[str, str]]:
	if except_import_error is not None:
		return []

	windows: List[Dict[str, str]] = []

	def _enum_handler(hwnd: int, ctx) -> None:
		if not win32gui.IsWindowVisible(hwnd):
			return
		title = win32gui.GetWindowText(hwnd)
		if not title:
			return
		try:
			pid = win32process.GetWindowThreadProcessId(hwnd)[1]
			pname = psutil.Process(pid).name()
		except Exception:
			pname = ""
		try:
			wclass = win32gui.GetClassName(hwnd)
		except Exception:
			wclass = ""
		windows.append({
			"hwnd": str(hwnd),
			"title": title,
			"process": pname,
			"class": wclass,
		})

	win32gui.EnumWindows(_enum_handler, None)
	# Sort by process then title
	windows.sort(key=lambda w: (w.get("process") or "", w.get("title") or ""))
	return windows


@app.get("/api/foreground")
async def api_foreground() -> Dict[str, Optional[str]]:
	return _get_foreground_info()


@app.post("/api/focus")
async def api_focus(payload: Dict[str, str]) -> JSONResponse:
	if except_import_error is not None:
		return JSONResponse({"ok": False, "message": "win32 API not available"}, status_code=400)
	try:
		hwnd = int(str(payload.get("hwnd", "0")))
		if hwnd <= 0:
			return JSONResponse({"ok": False, "message": "invalid hwnd"}, status_code=400)
		try:
			if win32con is not None:
				win32gui.ShowWindow(hwnd, win32con.SW_SHOWNORMAL)
			win32gui.SetForegroundWindow(hwnd)
		except Exception as e:
			return JSONResponse({"ok": False, "message": str(e)}, status_code=400)
		return JSONResponse({"ok": True})
	except Exception as e:
		return JSONResponse({"ok": False, "message": str(e)}, status_code=400)


@app.get("/api/target")
async def api_get_target() -> Dict[str, Optional[str]]:
	return {"mode": target_config.mode, "value": target_config.value}


@app.post("/api/target")
async def api_set_target(payload: Dict[str, str]) -> Dict[str, Optional[str]]:
	mode = (payload.get("mode") or "disabled").lower()
	value = payload.get("value")
	if mode not in {"disabled", "title", "process", "hwnd", "class", "all"}:
		raise ValueError("Invalid mode")
	# Clear current state on target change
	changed = False
	if state.clear_all():
		changed = True
	target_config.mode = mode
	target_config.value = value
	if changed and event_loop is not None:
		await manager.broadcast({"keys": []})
	return {"mode": target_config.mode, "value": target_config.value}


@app.get("/api/config")
async def api_get_config() -> Dict[str, int]:
	return {"port": app_config.port}


@app.get("/api/launcher-language")
async def api_get_launcher_language() -> Dict[str, str]:
	"""런처의 언어 설정을 가져옵니다."""
	try:
		# 런처 설정 파일에서 언어 정보를 읽어옵니다
		launcher_config_path = _config_dir() / "launcher_config.json"
		if launcher_config_path.exists():
			with open(launcher_config_path, 'r', encoding='utf-8') as f:
				launcher_config = json.load(f)
				language = launcher_config.get("language", "ko")
				return {"language": language}
	except Exception:
		pass
	
	# 기본값 반환
	return {"language": "ko"}


@app.post("/api/config")
async def api_set_config(payload: Dict[str, int]) -> JSONResponse:
	port = int(payload.get("port", app_config.port))
	if port < 1000 or port > 65535:
		return JSONResponse({"ok": False, "message": "포트는 1000-65535 범위여야 합니다."}, status_code=400)
	app_config.port = port
	save_app_config(app_config)
	return JSONResponse({"ok": True, "message": "저장됨. 서버 재시작 후 적용됩니다.", "port": app_config.port})


@app.get("/api/overlay-config")
async def api_get_overlay_config() -> Dict[str, object]:
	return asdict(app_config.overlay)


@app.post("/api/overlay-config")
async def api_set_overlay_config(payload: Dict[str, object]) -> JSONResponse:
	try:
		ov = app_config.overlay
		ov.fade_in_ms = max(0, int(payload.get("fade_in_ms", ov.fade_in_ms)))
		ov.fade_out_ms = max(0, int(payload.get("fade_out_ms", ov.fade_out_ms)))
		ov.chip_bg = str(payload.get("chip_bg", ov.chip_bg))
		ov.chip_fg = str(payload.get("chip_fg", ov.chip_fg))
		ov.chip_gap = max(0, int(payload.get("chip_gap", ov.chip_gap)))
		ov.chip_pad_v = max(0, int(payload.get("chip_pad_v", ov.chip_pad_v)))
		ov.chip_pad_h = max(0, int(payload.get("chip_pad_h", ov.chip_pad_h)))
		ov.chip_radius = max(0, int(payload.get("chip_radius", ov.chip_radius)))
		ov.chip_font_px = max(8, int(payload.get("chip_font_px", ov.chip_font_px)))
		ov.chip_font_weight = max(100, int(payload.get("chip_font_weight", ov.chip_font_weight)))
		ov.background = str(payload.get("background", ov.background))
		ov.cols = max(1, int(payload.get("cols", ov.cols)))
		ov.rows = max(0, int(payload.get("rows", ov.rows)))
		ov.single_line = bool(payload.get("single_line", ov.single_line))
		ov.single_line_scale = max(50, min(120, int(payload.get("single_line_scale", ov.single_line_scale))))
		align = str(payload.get("align", ov.align)).lower()
		direction = str(payload.get("direction", ov.direction)).lower()
		if align not in {"left", "center", "right"}:
			align = "center"
		if direction not in {"ltr", "rtl"}:
			direction = "ltr"
		ov.align = align
		ov.direction = direction
		save_app_config(app_config)
		# Broadcast to overlays
		await manager.broadcast({"type": "config", "overlay": asdict(ov)})
		return JSONResponse({"ok": True})
	except Exception as e:
		return JSONResponse({"ok": False, "message": str(e)}, status_code=400)


# -------------------------------
# Lifecycle
# -------------------------------
@app.on_event("startup")
async def on_startup() -> None:
	global event_loop
	event_loop = asyncio.get_event_loop()
	# Start threads safely
	try:
		if not kb_thread.is_alive():
			kb_thread.start()
	except RuntimeError:
		pass
	try:
		if not focus_thread.is_alive():
			focus_thread.start()
	except RuntimeError:
		pass


@app.on_event("shutdown")
async def on_shutdown() -> None:
	try:
		kb_thread.stop()
		focus_thread.stop()
		# Give threads a moment to exit
		time.sleep(0.1)
		# Join if possible
		if kb_thread.is_alive():
			try: kb_thread.join(timeout=0.5)
			except Exception: pass
		if focus_thread.is_alive():
			try: focus_thread.join(timeout=0.5)
			except Exception: pass
	except Exception:
		pass


if __name__ == "__main__":
	# Use configured port when running directly
	uvicorn.run(app, host="127.0.0.1", port=load_app_config().port)