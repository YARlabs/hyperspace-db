import axios from "axios"

export const api = axios.create({
    baseURL: "/api",
})

export const setAuthToken = (token: string) => {
    api.defaults.headers.common["x-api-key"] = token
}

api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            // Only clear if we're not on login page already
            if (localStorage.getItem("hyperspace_api_key") && !window.location.pathname.includes("/login")) {
                localStorage.removeItem("hyperspace_api_key")
                window.location.href = "/login"
            }
        }
        return Promise.reject(error)
    }
)

const token = localStorage.getItem("hyperspace_api_key")
if (token) setAuthToken(token)

export const fetchStatus = async () => {
    const res = await api.get("/status")
    return res.data
}

export const fetchHealth = async () => {
    const res = await api.get("/health")
    return res.data
}

export const fetchMetrics = async () => {
    const res = await api.get("/metrics")
    return res.data
}

export const fetchCollections = async () => {
    const res = await api.get("/collections")
    return res.data
}

export const insertVector = async (name: string, data: any) => {
    const res = await api.post(`/collections/${name}/insert`, data)
    return res.data
}

export const searchCollection = async (name: string, payload: any) => {
    const res = await api.post(`/collections/${name}/search`, payload)
    return res.data
}

export const searchBatch = async (name: string, payload: any) => {
    const res = await api.post(`/collections/${name}/search/batch`, payload)
    return res.data
}

export const searchMulti = async (payload: any) => {
    const res = await api.post(`/search/multi`, payload)
    return res.data
}

export const scrollCollection = async (name: string, payload: any) => {
    const res = await api.post(`/collections/${name}/scroll`, payload)
    return res.data
}

export const countFiltered = async (name: string, payload: any) => {
    const res = await api.post(`/collections/${name}/count`, payload)
    return res.data
}

export const updatePayload = async (name: string, id: number, metadata: any) => {
    const res = await api.post(`/collections/${name}/payload`, { id, metadata })
    return res.data
}

export const getCollectionStats = (name: string) => api.get(`/collections/${name}/stats`).then(r => r.data)

export const updateCollectionConfig = (name: string, config: any) => api.patch(`/collections/${name}/config`, config)

// Cache management — L0 Hot Tier
export const getCacheStats = (name: string) => api.get(`/collections/${name}/cache/stats`).then(r => r.data)

export const clearCache = (name: string) => api.post(`/collections/${name}/cache/clear`).then(r => r.data)

export const updateCacheConfig = (name: string, policy: string, annThreshold?: number | null) =>
    api.post(`/collections/${name}/cache/config`, {
        policy,
        ann_threshold: annThreshold ?? undefined
    }).then(r => r.data)

export const freezeCollection = async (name: string) => {
    const res = await api.post(`/collections/${name}/freeze`)
    return res.data
}

export const unfreezeCollection = async (name: string) => {
    const res = await api.post(`/collections/${name}/unfreeze`)
    return res.data
}

export const getPoints = async (name: string, ids: number[]) => {
    const res = await api.get(`/collections/${name}/points`, { params: { ids: ids.join(',') } })
    return res.data
}

export const deletePoint = async (name: string, id: number) => {
    const res = await api.delete(`/collections/${name}/points/${id}`)
    return res.data
}

export const fetchTrajectoryHistory = async () => {
    const res = await api.get("/admin/trajectory/history")
    return res.data
}

export const fetchAgentRuns = async () => {
    const res = await api.get("/admin/runs")
    return res.data
}

export const fetchAgentRunById = async (sessionId: string) => {
    const res = await api.get(`/admin/runs/${sessionId}`)
    return res.data
}

export const startMigrationService = async () => {
    const res = await api.post("/admin/migration/start")
    return res.data
}

export const getMigrationEngineStatus = async () => {
    const res = await api.get("/admin/migration/status")
    return res.data
}

export const startMigrationTask = async (config: any) => {
    const res = await api.post("/admin/migration/task", config)
    return res.data
}

export const getMigrationTaskStatus = async (taskId: string) => {
    const res = await api.get(`/admin/migration/task/${taskId}`)
    return res.data
}

export const deleteCollection = async (name: string) => {
    const res = await api.delete(`/collections/${name}`)
    return res.data
}

export const triggerVacuum = async () => {
    const res = await api.post("/admin/vacuum")
    return res.data
}

export const fetchEcoMetrics = async (range: string) => {
    const res = await api.get(`/eco/metrics`, { params: { range } })
    return res.data
}

export const fetchEsgReportBlob = async (range: string, format: "json" | "csv") => {
    const res = await api.get(`/eco/esg-report`, {
        params: { range, format },
        responseType: 'blob'
    })
    return res.data
}

export const fetchLogs = async () => {
    const res = await api.get("/logs")
    return res.data
}

export const grantCollectionAccess = async (name: string, granteeId: string, privilege: string) => {
    const res = await api.post(`/collections/${name}/grant`, { grantee_id: granteeId, privilege })
    return res.data
}

export const revokeCollectionAccess = async (name: string, granteeId: string) => {
    const res = await api.post(`/collections/${name}/revoke`, { grantee_id: granteeId })
    return res.data
}

export const listCollectionGrants = async (name: string) => {
    const res = await api.get(`/collections/${name}/grants`)
    return res.data
}

// Hyperspace PDK Extensions & Plugins Management (v4.2.0)
export interface ExtensionProcedureSchema {
    name: string
    description: string
    requires_collection: boolean
    input_schema: Record<string, any>
    output_schema: Record<string, any>
}

export interface ExtensionManifest {
    name: string
    version: string
    author: string
    description: string
    runtime: string
    enabled: boolean
    source: string
    hooks: string[]
    procedures: ExtensionProcedureSchema[]
    custom_metrics: Array<{
        name: string
        description: string
        is_symmetric: boolean
        preserves_unit_norm: boolean
    }>
}

export interface ExtensionsCatalogResponse {
    pdk_version: string
    pdk_api_version: number
    plugins_dir: string
    supported_metrics: string[]
    extensions: ExtensionManifest[]
}

export const fetchExtensionsCatalog = async (): Promise<ExtensionsCatalogResponse> => {
    const res = await api.get("/extensions")
    return res.data
}

export const enableExtension = async (plugin: string) => {
    const res = await api.post(`/extensions/${plugin}/enable`)
    return res.data
}

export const disableExtension = async (plugin: string) => {
    const res = await api.post(`/extensions/${plugin}/disable`)
    return res.data
}

export const reloadExtensions = async () => {
    const res = await api.post("/extensions/reload")
    return res.data
}

export const callExtensionProcedure = async (
    plugin: string,
    method: string,
    args: Record<string, any>,
    collection?: string
) => {
    if (collection && collection.trim() !== "") {
        const res = await api.post(`/collections/${collection.trim()}/ext/${plugin}/${method}`, args)
        return res.data
    }
    const res = await api.post(`/ext/${plugin}/${method}`, args)
    return res.data
}


