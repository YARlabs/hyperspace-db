import { useState, useEffect } from 'react'
// @ts-ignore
import logo from '/hyperspace.svg'
import './App.css'

function App() {
  const [collections, setCollections] = useState<string[]>([])

  useEffect(() => {
    fetch('/api/collections')
      .then(res => res.json())
      .then(data => setCollections(data))
      .catch(err => console.error(err))
  }, [])

  return (
    <>
      <div>
        <a href="#" target="_blank">
          <img src={logo} className="logo" alt="HyperspaceDB Logo" />
        </a>
      </div>
      <h1>HyperspaceDB Dashboard</h1>
      <div className="card">
        <h2>Collections</h2>
        <ul>
          {collections.map(c => <li key={c}>{c}</li>)}
        </ul>
        {collections.length === 0 && <p>No collections found.</p>}
      </div>
    </>
  )
}

export default App
