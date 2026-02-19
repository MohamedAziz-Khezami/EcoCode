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

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || ''

export interface RunSummary extends Omit<Run, 'records'> {}
export interface RunDetail extends Run {}
export interface Improvement {
  runId: string
  previousRunId: string
  energyChange: number
  carbonChange: number
  waterChange: number
  timestamp: Date
}

/**
 * Fetch all runs with summary metrics
 * 
 * TODO: When backend is ready, replace the `/api/runs` call with:
 * return fetch(`${BACKEND_URL}/runs`).then(res => res.json())
 */
export async function fetchAllRuns(): Promise<RunSummary[]> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/runs`)
    if (!response.ok) {
      throw new Error(`Failed to fetch runs: ${response.statusText}`)
    }
    return response.json()
  } catch (error) {
    console.error('Error fetching runs:', error)
    throw error
  }
}

/**
 * Fetch a specific run with all detail records
 * 
 * TODO: When backend is ready, replace with:
 * return fetch(`${BACKEND_URL}/runs/${id}`).then(res => res.json())
 */
export async function fetchRunDetail(id: string): Promise<RunDetail> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/runs/${id}`)
    if (!response.ok) {
      throw new Error(`Failed to fetch run detail: ${response.statusText}`)
    }
    return response.json()
  } catch (error) {
    console.error(`Error fetching run ${id}:`, error)
    throw error
  }
}

/**
 * Fetch comparison data between runs
 * 
 * TODO: When backend is ready, replace with:
 * return fetch(`${BACKEND_URL}/runs/compare?ids=${ids.join(',')}`).then(res => res.json())
 */
export async function fetchImprovements(): Promise<Improvement[]> {
  try {
    const response = await fetch(`${API_BASE_URL}/api/improvements`)
    if (!response.ok) {
      throw new Error(`Failed to fetch improvements: ${response.statusText}`)
    }
    return response.json()
  } catch (error) {
    console.error('Error fetching improvements:', error)
    throw error
  }
}

/**
 * Create a new run (for future use with backend)
 * 
 * TODO: Implement when backend is ready:
 * return fetch(`${BACKEND_URL}/runs`, {
 *   method: 'POST',
 *   headers: { 'Content-Type': 'application/json' },
 *   body: JSON.stringify(data)
 * }).then(res => res.json())
 */
export async function createRun(data: Partial<Run>): Promise<Run> {
  throw new Error('Not yet implemented - awaiting backend integration')
}

/**
 * Update an existing run (for future use with backend)
 * 
 * TODO: Implement when backend is ready:
 * return fetch(`${BACKEND_URL}/runs/${id}`, {
 *   method: 'PATCH',
 *   headers: { 'Content-Type': 'application/json' },
 *   body: JSON.stringify(data)
 * }).then(res => res.json())
 */
export async function updateRun(id: string, data: Partial<Run>): Promise<Run> {
  throw new Error('Not yet implemented - awaiting backend integration')
}

/**
 * Delete a run (for future use with backend)
 * 
 * TODO: Implement when backend is ready:
 * return fetch(`${BACKEND_URL}/runs/${id}`, {
 *   method: 'DELETE'
 * })
 */
export async function deleteRun(id: string): Promise<void> {
  throw new Error('Not yet implemented - awaiting backend integration')
}

/**
 * Search/filter runs with query parameters
 * 
 * TODO: When backend is ready, replace with:
 * const params = new URLSearchParams(filters)
 * return fetch(`${BACKEND_URL}/runs?${params}`).then(res => res.json())
 */
export async function searchRuns(filters: Record<string, any>): Promise<RunSummary[]> {
  try {
    // For now, fetch all runs and filter on client
    // In production, this should be done server-side for better performance
    const allRuns = await fetchAllRuns()
    
    // TODO: Replace with server-side filtering when backend is ready
    return allRuns.filter(run => {
      if (filters.status && run.status !== filters.status) return false
      if (filters.minEnergy && run.totalEnergy < filters.minEnergy) return false
      if (filters.maxEnergy && run.totalEnergy > filters.maxEnergy) return false
      return true
    })
  } catch (error) {
    console.error('Error searching runs:', error)
    throw error
  }
}

/**
 * Batch delete multiple runs
 * 
 * TODO: When backend is ready, use batch endpoint:
 * return fetch(`${BACKEND_URL}/runs/batch`, {
 *   method: 'DELETE',
 *   headers: { 'Content-Type': 'application/json' },
 *   body: JSON.stringify({ ids })
 * })
 */
export async function batchDeleteRuns(ids: string[]): Promise<void> {
  try {
    // Currently calls delete for each run
    // TODO: Use batch endpoint in backend for better performance
    await Promise.all(ids.map(id => deleteRun(id)))
  } catch (error) {
    console.error('Error batch deleting runs:', error)
    throw error
  }
}

/**
 * Export run data to CSV
 * 
 * TODO: When backend is ready, use server-side export:
 * return fetch(`${BACKEND_URL}/runs/${id}/export?format=csv`)
 *   .then(res => res.blob())
 */
export async function exportRunAsCSV(id: string): Promise<Blob> {
  try {
    const run = await fetchRunDetail(id)
    
    // TODO: Replace with backend-generated CSV when available
    // This is a basic client-side implementation
    const headers = ['Timestamp', 'CPU Usage (%)', 'CPU Energy (W)', 'GPU Usage (%)', 'GPU Energy (W)']
    const rows = run.records.map(record => [
      new Date(record.timestamp).toISOString(),
      record.cpuUsage,
      record.cpuEnergy,
      record.gpuUsage,
      record.gpuEnergy
    ])
    
    const csv = [headers, ...rows].map(row => row.join(',')).join('\n')
    return new Blob([csv], { type: 'text/csv' })
  } catch (error) {
    console.error('Error exporting run:', error)
    throw error
  }
}
