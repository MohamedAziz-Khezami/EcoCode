// EcoCode Record types
export interface Record {
  id: string
  pid: number
  timestamp: number
  cpu_usage: number // percentage 0-100
  cpu_energy: number // watts
  gpu_usage: number // percentage 0-100
  gpu_energy: number // watts
}

export interface Run {
  id: string
  name: string
  timestamp: number
  status: 'running' | 'finished' | 'failed'
  totalEnergy: number // Wh
  totalCoreHours: number
  avgCpuUsage: number // percentage
  avgGpuUsage: number // percentage
  carbonFootprint: number // gCO2
  waterConsumption: number // mL
  duration: number // seconds
  records: Record[]
}

// Constants for calculations
const CARBON_FACTOR = 0.475 // kgCO2/kWh (world average)
const WATER_FACTOR = 1.8 // L/kWh

// Helper to generate mock records for a run
function generateRunRecords(count: number, startTime: number): Record[] {
  const records: Record[] = []
  const timeStep = 60000 // 1 minute intervals in ms

  for (let i = 0; i < count; i++) {
    records.push({
      id: `record-${i}`,
      pid: Math.floor(Math.random() * 65535),
      timestamp: startTime + i * timeStep,
      cpu_usage: 20 + Math.random() * 70,
      cpu_energy: 50 + Math.random() * 150,
      gpu_usage: Math.random() > 0.3 ? 10 + Math.random() * 80 : 0,
      gpu_energy: Math.random() > 0.3 ? 30 + Math.random() * 200 : 0,
    })
  }

  return records
}

// Helper to calculate run metrics from records
function calculateRunMetrics(records: Record[]) {
  const totalCpuEnergy = records.reduce((sum, r) => sum + r.cpu_energy, 0)
  const totalGpuEnergy = records.reduce((sum, r) => sum + r.gpu_energy, 0)
  const totalEnergy = totalCpuEnergy + totalGpuEnergy

  const avgCpuUsage =
    records.reduce((sum, r) => sum + r.cpu_usage, 0) / records.length
  const avgGpuUsage =
    records.reduce((sum, r) => sum + r.gpu_usage, 0) / records.length

  // Convert Wh to kWh then apply factors
  const energyKwh = totalEnergy / 1000
  const carbonFootprint = energyKwh * CARBON_FACTOR * 1000 // gCO2
  const waterConsumption = energyKwh * WATER_FACTOR * 1000 // mL

  return {
    totalEnergy,
    avgCpuUsage,
    avgGpuUsage,
    carbonFootprint,
    waterConsumption,
  }
}

// Generate 10 mock runs
export function generateMockRuns(): Run[] {
  const now = Date.now()
  const runs: Run[] = []

  for (let i = 0; i < 10; i++) {
    const recordCount = 20 + Math.floor(Math.random() * 40)
    const records = generateRunRecords(recordCount, now - (10 - i) * 86400000)

    const metrics = calculateRunMetrics(records)

    runs.push({
      id: `run-${i + 1}`,
      name: `Training Run ${i + 1}`,
      timestamp: now - (10 - i) * 86400000,
      status: i < 2 ? 'running' : i === 2 ? 'failed' : 'finished',
      totalEnergy: metrics.totalEnergy,
      totalCoreHours: (metrics.totalEnergy / 1000) * 24, // Rough estimate
      avgCpuUsage: metrics.avgCpuUsage,
      avgGpuUsage: metrics.avgGpuUsage,
      carbonFootprint: metrics.carbonFootprint,
      waterConsumption: metrics.waterConsumption,
      duration: recordCount * 60, // 1 minute per record
      records,
    })
  }

  return runs.reverse()
}

// Get all runs with summary metrics (no records)
export function getRuns(): Omit<Run, 'records'>[] {
  const runs = generateMockRuns()
  return runs.map(({ records, ...rest }) => rest)
}

// Get a single run with all details
export function getRunById(id: string): Run | null {
  const runs = generateMockRuns()
  return runs.find((r) => r.id === id) || null
}

// Get comparison data between consecutive runs
export function getImprovements(): Record[] {
  const runs = getRuns()
  const improvements: Record[] = []

  for (let i = 1; i < runs.length; i++) {
    const prev = runs[i - 1]
    const curr = runs[i]

    improvements.push({
      id: `improvement-${i}`,
      pid: 0,
      timestamp: curr.timestamp,
      cpu_usage:
        ((prev.avgCpuUsage - curr.avgCpuUsage) / prev.avgCpuUsage) * 100,
      cpu_energy: ((prev.totalEnergy - curr.totalEnergy) / prev.totalEnergy) * 100,
      gpu_usage:
        ((prev.avgGpuUsage - curr.avgGpuUsage) / prev.avgGpuUsage) * 100,
      gpu_energy:
        ((prev.carbonFootprint - curr.carbonFootprint) /
          prev.carbonFootprint) *
        100,
    })
  }

  return improvements
}
