/**
 * API Client for EcoCode Dashboard
 * 
 * This service layer abstracts data fetching and makes it easy to swap
 * between mock data and real backend calls.
 * 
 * To integrate with a real backend:
 * 1. Replace the mock data imports with actual API calls
 * 2. Update the fetch URLs from `/api/*` to your backend endpoint
 * 3. Add error handling and retry logic as needed
 */

import { Run, Record } from './mock-data'

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001'

// --- Internal Mappings ---

const CARBON_FACTOR = 0.475 // kgCO2/kWh
const WATER_FACTOR = 1.8 // L/kWh

interface BackendProject {
  id: number
  name: string
}

interface BackendRun {
  id: number
  name: string
  project_id: number
}

interface BackendRecordPoint {
  id: number
  run_id: number
  pid: number
  timestamp: string
  cpu_usage: number
  cpu_energy: number
  gpu_usage: number
  gpu_energy: number
  mem_usage: number
  mem_energy: number
  igpu_usage: number
  igpu_energy: number
}

interface BackendRunSummary {
  run_id: number
  total_cpu_energy: number
  total_gpu_energy: number
  total_mem_energy: number
  total_igpu_energy: number
}

function mapRecord(r: BackendRecordPoint): Record {
  return {
    id: r.id.toString(),
    pid: r.pid,
    timestamp: new Date(r.timestamp).getTime(),
    cpu_usage: r.cpu_usage,
    cpu_energy: r.cpu_energy,
    gpu_usage: r.gpu_usage,
    gpu_energy: r.gpu_energy,
    mem_usage: r.mem_usage,
    mem_energy: r.mem_energy,
    igpu_usage: r.igpu_usage,
    igpu_energy: r.igpu_energy,
  }
}

async function mapRun(backendRun: BackendRun): Promise<Run> {
  // Fetch summary for metrics
  const summaryRes = await fetch(`${API_BASE_URL}/api/run/${backendRun.id}/summary`)
  const summary: BackendRunSummary | null = summaryRes.ok ? await summaryRes.json() : null

  // Fetch record points for charts (optional depending on if we need them all upfront)
  // For now, let's just fetch them to match the expected interface of fetchRunDetail
  const recordsRes = await fetch(`${API_BASE_URL}/api/run/${backendRun.id}/record_points`)
  const backendRecords: BackendRecordPoint[] = recordsRes.ok ? await recordsRes.json() : []
  const records = backendRecords.map(mapRecord)

  const totalEnergy = summary ? (summary.total_cpu_energy + summary.total_gpu_energy + summary.total_mem_energy + summary.total_igpu_energy) : 0
  const energyKwh = totalEnergy / 1000

  // Calculate averages from records if summary doesn't have them
  const avgCpuUsage = records.length > 0 ? records.reduce((sum, r) => sum + r.cpu_usage, 0) / records.length : 0
  const avgGpuUsage = records.length > 0 ? records.reduce((sum, r) => sum + r.gpu_usage, 0) / records.length : 0
  const avgMemUsage = records.length > 0 ? records.reduce((sum, r) => sum + r.mem_usage, 0) / records.length : 0

  return {
    id: backendRun.id.toString(),
    name: backendRun.name,
    timestamp: records.length > 0 ? records[0].timestamp : Date.now(),
    status: 'finished', // Backend doesn't explicitly expose status yet
    totalEnergy,
    totalCoreHours: energyKwh * 24, // Mock calculation
    avgCpuUsage,
    avgGpuUsage,
    avgMemUsage,
    carbonFootprint: energyKwh * CARBON_FACTOR * 1000,
    waterConsumption: energyKwh * WATER_FACTOR * 1000,
    duration: records.length * 60, // Mock duration
    records,
  }
}

// --- Public API ---

export interface RunSummary extends Omit<Run, 'records'> { }
export interface RunDetail extends Run { }
export interface Improvement {
  id: string
  pid: number
  timestamp: number
  cpu_usage: number
  cpu_energy: number
  gpu_usage: number
  gpu_energy: number
  mem_usage: number
  mem_energy: number
  igpu_usage: number
  igpu_energy: number
}

/**
 * Fetch all runs for all projects
 */
export async function fetchAllRuns(): Promise<RunSummary[]> {
  try {
    const projectsRes = await fetch(`${API_BASE_URL}/api/projects`)
    if (!projectsRes.ok) throw new Error('Failed to fetch projects')
    const projects: BackendProject[] = await projectsRes.json()

    const allRuns: RunSummary[] = []

    for (const project of projects) {
      const runsRes = await fetch(`${API_BASE_URL}/api/project/${project.id}/runs`)
      if (runsRes.ok) {
        const backendRuns: BackendRun[] = await runsRes.json()
        for (const br of backendRuns) {
          // In a real app we might not want to fetch full details here, 
          // but fetchAllRuns needs the metrics for the metric cards.
          const run = await mapRun(br)
          const { records, ...summary } = run
          allRuns.push(summary)
        }
      }
    }

    return allRuns.sort((a, b) => b.timestamp - a.timestamp)
  } catch (error) {
    console.error('Error fetching runs:', error)
    throw error
  }
}

/**
 * Fetch a specific run with record points
 */
export async function fetchRunDetail(id: string): Promise<RunDetail> {
  try {
    // We need to find the run name and project_id, but the API doesn't have GET /api/run/{id}
    // We'll have to find it in the list or assume it's reachable.
    // Hack: Just get the data we can.
    const runRes = await fetch(`${API_BASE_URL}/api/projects`) // Dummy to get started if needed

    // Actually, let's just use the ID to fetch what we need
    const summaryRes = await fetch(`${API_BASE_URL}/api/run/${id}/summary`)
    if (!summaryRes.ok) throw new Error(`Run ${id} not found`)
    const summary: BackendRunSummary = await summaryRes.json()

    const recordsRes = await fetch(`${API_BASE_URL}/api/run/${id}/record_points`)
    const backendRecords: BackendRecordPoint[] = recordsRes.ok ? await recordsRes.json() : []
    const records = backendRecords.map(mapRecord)

    const totalEnergy = summary.total_cpu_energy + summary.total_gpu_energy + summary.total_mem_energy + summary.total_igpu_energy
    const energyKwh = totalEnergy / 1000
    const avgCpuUsage = records.length > 0 ? records.reduce((sum, r) => sum + r.cpu_usage, 0) / records.length : 0
    const avgGpuUsage = records.length > 0 ? records.reduce((sum, r) => sum + r.gpu_usage, 0) / records.length : 0
    const avgMemUsage = records.length > 0 ? records.reduce((sum, r) => sum + r.mem_usage, 0) / records.length : 0

    return {
      id,
      name: `Run ${id}`,
      timestamp: records.length > 0 ? records[0].timestamp : Date.now(),
      status: 'finished',
      totalEnergy,
      totalCoreHours: energyKwh * 24,
      avgCpuUsage,
      avgGpuUsage,
      avgMemUsage,
      carbonFootprint: energyKwh * CARBON_FACTOR * 1000,
      waterConsumption: energyKwh * WATER_FACTOR * 1000,
      duration: records.length * 60,
      records,
    }
  } catch (error) {
    console.error(`Error fetching run ${id}:`, error)
    throw error
  }
}

/**
 * Fetch improvements (placeholder as backend doesn't have this yet)
 */
export async function fetchImprovements(): Promise<Improvement[]> {
  return [] // Return empty for now
}

export async function createRun(data: Partial<Run>): Promise<Run> {
  throw new Error('Not implemented')
}

export async function updateRun(id: string, data: Partial<Run>): Promise<Run> {
  throw new Error('Not implemented')
}

export async function deleteRun(id: string): Promise<void> {
  throw new Error('Not implemented')
}

export async function searchRuns(filters: Record<string, any>): Promise<RunSummary[]> {
  const allRuns = await fetchAllRuns()
  return allRuns.filter(run => {
    if (filters.status && run.status !== filters.status) return false
    return true
  })
}

export async function batchDeleteRuns(ids: string[]): Promise<void> {
  await Promise.all(ids.map(id => deleteRun(id)))
}

export async function exportRunAsCSV(id: string): Promise<Blob> {
  const run = await fetchRunDetail(id)
  const headers = ['Timestamp', 'CPU Usage (%)', 'CPU Energy (W)', 'GPU Usage (%)', 'GPU Energy (W)', 'MEM Energy (W)', 'iGPU Energy (W)']
  const rows = run.records.map(record => [
    new Date(record.timestamp).toISOString(),
    record.cpu_usage,
    record.cpu_energy,
    record.gpu_usage,
    record.gpu_energy,
    record.mem_energy,
    record.igpu_energy,
  ])
  const csv = [headers, ...rows].map(row => row.join(',')).join('\n')
  return new Blob([csv], { type: 'text/csv' })
}
