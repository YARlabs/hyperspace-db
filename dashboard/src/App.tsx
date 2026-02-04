import { useState } from 'react'
// @ts-ignore
import logo from '/hyperspace.svg'
import './App.css'
import Login from './components/Login'
import Dashboard from './components/Dashboard'

function App() {
  const [apiKey, setApiKey] = useState<string | null>(localStorage.getItem('hyperspace_api_key'))

  const handleLogin = (key: string) => {
    localStorage.setItem('hyperspace_api_key', key)
    setApiKey(key)
  }

  const handleLogout = () => {
    localStorage.removeItem('hyperspace_api_key')
    setApiKey(null)
  }

  if (!apiKey) {
    return <Login onLogin={handleLogin} logo={logo} />
  }

  return <Dashboard apiKey={apiKey} onLogout={handleLogout} logo={logo} />
}

export default App
