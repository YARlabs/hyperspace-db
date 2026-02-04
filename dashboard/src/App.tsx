import { useState, useEffect } from 'react'
// @ts-ignore
import logo from '/hyperspace.svg'
import './App.css'

function App() {
  const [collections, setCollections] = useState<string[]>([])

  const [newColName, setNewColName] = useState('')
  const [newColDim, setNewColDim] = useState('1024')
  const [newColMetric] = useState('poincare')

  const fetchCollections = () => {
    fetch('/api/collections')
      .then(res => res.json())
      .then(data => setCollections(data))
      .catch(err => console.error(err))
  }

  useEffect(() => {
    fetchCollections()
    const interval = setInterval(fetchCollections, 2000)
    return () => clearInterval(interval)
  }, [])

  const createCollection = async () => {
    await fetch('/api/collections', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: newColName, dimension: parseInt(newColDim), metric: newColMetric })
    })
    setNewColName('')
    fetchCollections()
  }

  const deleteCollection = async (name: string) => {
    if (!confirm(`Delete ${name}?`)) return
    await fetch(`/api/collections/${name}`, { method: 'DELETE' })
    fetchCollections()
  }

  return (
    <>
      <div>
        <a href="#" target="_blank">
          <img src={logo} className="logo" alt="HyperspaceDB Logo" />
        </a>
      </div>
      <h1>HyperspaceDB Dashboard</h1>

      <div className="card">
        <h2>Create Collection</h2>
        <div style={{ display: 'flex', gap: '10px', justifyContent: 'center' }}>
          <input
            placeholder="Name"
            value={newColName}
            onChange={e => setNewColName(e.target.value)}
          />
          <select value={newColDim} onChange={e => setNewColDim(e.target.value)}>
            <option value="8">8D (Test)</option>
            <option value="768">768D (BERT)</option>
            <option value="1024">1024D (BGE-M3)</option>
            <option value="1536">1536D (OpenAI)</option>
          </select>
          <button onClick={createCollection}>Create</button>
        </div>
      </div>

      <div className="card">
        <h2>Collections</h2>
        <ul style={{ listStyle: 'none', padding: 0 }}>
          {collections.map(c => (
            <li key={c} style={{ margin: '10px 0', border: '1px solid #333', padding: '10px', borderRadius: '8px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span>{c}</span>
              <button onClick={() => deleteCollection(c)} style={{ backgroundColor: '#ff4444' }}>Delete</button>
            </li>
          ))}
        </ul>
        {collections.length === 0 && <p>No collections found.</p>}
      </div>
    </>
  )
}

export default App
