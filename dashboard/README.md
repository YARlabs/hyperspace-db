# HyperspaceDB Dashboard

Professional web-based management interface for HyperspaceDB.

## Features

### 🔐 Authentication
- API key-based access control
- Secure SHA-256 hashing
- Default key: `I_LOVE_HYPERSPACEDB`

### 📊 Collection Management
- Create collections with preset configurations
- Delete collections
- View collection statistics
- Real-time collection list updates

#### Supported Presets
**Hyperbolic (Poincaré Metric)**
- 16D, 32D, 64D, 128D

**Euclidean (L2 Metric)**
- 1024D, 1536D, 2048D

### 🌀 Poincaré Disk Visualizer
- Interactive canvas-based visualization
- Hyperbolic vector space representation
- Real-time data distribution display
- Collection-specific views

### 📈 System Metrics
- Total collections count
- Total vectors indexed
- Memory usage monitoring
- Queries per second (QPS)
- Real-time history charts

## Development

### Prerequisites
- Node.js 18+
- npm or yarn

### Setup

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build
```

### Environment

The dashboard connects to the HyperspaceDB server at `http://localhost:50050` by default.

## Architecture

### Tech Stack
- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **Canvas API** - Poincaré visualization

### Components

- `Login.tsx` - Authentication screen
- `Dashboard.tsx` - Main layout with tabs
- `CollectionManager.tsx` - Collection CRUD operations
- `PoincareVisualizer.tsx` - Hyperbolic space visualization
- `SystemMetrics.tsx` - Real-time monitoring

## API Integration

The dashboard communicates with HyperspaceDB via HTTP REST API:

```typescript
// List collections
GET /api/collections
Headers: { 'x-api-key': 'YOUR_KEY' }

// Create collection
POST /api/collections
Headers: { 'x-api-key': 'YOUR_KEY', 'Content-Type': 'application/json' }
Body: { name: string, dimension: number, metric: string }

// Delete collection
DELETE /api/collections/{name}
Headers: { 'x-api-key': 'YOUR_KEY' }

// Get stats
GET /api/collections/{name}/stats
Headers: { 'x-api-key': 'YOUR_KEY' }
```

## Security

- All API requests require `x-api-key` header
- API keys are hashed with SHA-256 before comparison
- Keys are stored in browser localStorage
- No sensitive data in URLs or query parameters

## Customization

### Changing API Key

Set the `HYPERSPACE_API_KEY` environment variable in the server's `.env` file:

```bash
HYPERSPACE_API_KEY=your_custom_key_here
```

### Adding New Presets

Edit `CollectionManager.tsx`:

```typescript
const COLLECTION_PRESETS = [
  { label: 'Your Preset', dimension: 512, metric: 'poincare' },
  // ... existing presets
]
```

## Deployment

The dashboard is embedded in the HyperspaceDB server binary via `rust-embed`. The build artifacts are automatically included during server compilation.

To update the dashboard:

```bash
cd dashboard
npm run build
cd ..
cargo build --release
```

## License

Same as HyperspaceDB main project (AGPLv3 + Commercial).
