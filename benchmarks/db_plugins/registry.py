import importlib
import pkgutil
from typing import Dict, List

from db_plugins.base import DatabasePlugin


def load_plugins() -> Dict[str, DatabasePlugin]:
    plugins: Dict[str, DatabasePlugin] = {}
    package_name = "db_plugins.adapters"
    package = importlib.import_module(package_name)

    for _, module_name, _ in pkgutil.iter_modules(package.__path__):
        module = importlib.import_module(f"{package_name}.{module_name}")
        plugin = getattr(module, "PLUGIN", None)
        if plugin is None:
            continue
        if not isinstance(plugin, DatabasePlugin):
            continue
        if not plugin.name:
            continue
        plugins[plugin.name] = plugin
    return plugins


def select_plugins(all_plugins: Dict[str, DatabasePlugin], target_db: str | None) -> List[DatabasePlugin]:
    if not target_db:
        return list(all_plugins.values())

    norm = target_db.strip().lower()
    selected = []
    for name, plugin in all_plugins.items():
        if norm in name.lower():
            selected.append(plugin)
    return selected
