import { Routes, Route, Navigate } from "react-router-dom"
import { DashboardLayout } from "./layouts/DashboardLayout"
import { AuthProvider, RequireAuth } from "./hooks/use-auth"
import { AuthPage } from "./pages/AuthPage"
import { OverviewPage } from "./pages/OverviewPage"
import { CollectionsPage } from "./pages/CollectionsPage"
import { DataExplorerPage } from "./pages/DataExplorerPage"
import { NodesPage } from "./pages/NodesPage"
import { SettingsPage } from "./pages/SettingsPage"

function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route path="/login" element={<AuthPage />} />

        <Route element={
          <RequireAuth>
            <DashboardLayout />
          </RequireAuth>
        }>
          <Route path="/" element={<OverviewPage />} />
          <Route path="/collections" element={<CollectionsPage />} />
          <Route path="/nodes" element={<NodesPage />} />
          <Route path="/explorer" element={<DataExplorerPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Route>

        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </AuthProvider>
  )
}

export default App
