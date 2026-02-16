from abc import ABC, abstractmethod

from plugin_runtime import BenchmarkContext, Result


class DatabasePlugin(ABC):
    name: str = ""

    @abstractmethod
    def is_available(self) -> bool:
        raise NotImplementedError

    @abstractmethod
    def run(self, ctx: BenchmarkContext) -> Result:
        raise NotImplementedError
