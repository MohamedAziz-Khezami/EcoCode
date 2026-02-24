'use client'

import { useEffect, useState } from 'react'
import {
  Zap,
  Leaf,
  Droplets,
  Clock,
  Activity,
} from 'lucide-react'
import { DashboardLayout } from '@/components/DashboardLayout'
import { MetricCard } from '@/components/MetricCard'
import { EnergyChart } from '@/components/EnergyChart'
import { UtilizationChart } from '@/components/UtilizationChart'
import { RecentRunsTable } from '@/components/RecentRunsTable'
import { Run } from '@/lib/mock-data'
import { fetchAllRuns, fetchRunDetail } from '@/lib/api-client'
import { useSSE } from '@/hooks/use-sse'

export default function Dashboard() {
  const [runs, setRuns] = useState<Omit<Run, 'records'>[]>([])
  const [currentRun, setCurrentRun] = useState<Run | null>(null)
  const [loading, setLoading] = useState(true)
  const { latestRecord } = useSSE()

  // Update current run with real-time SSE data
  useEffect(() => {
    if (!latestRecord) return

    setCurrentRun((prevRun) => {
      if (!prevRun) return null

      // Check if the record already exists to avoid duplicates
      if (prevRun.records.some((r) => r.id === latestRecord.id)) return prevRun

      const updatedRecords = [...prevRun.records, latestRecord]
      const count = updatedRecords.length

      // Calculate total values incrementally if possible, but for simplicity here we re-sum lightly
      // Total energy across all components
      const totalCpuEnergy = updatedRecords.reduce((sum, r) => sum + r.cpu_energy, 0)
      const totalGpuEnergy = updatedRecords.reduce((sum, r) => sum + r.gpu_energy, 0)
      const totalMemEnergy = updatedRecords.reduce((sum, r) => sum + r.mem_energy, 0)
      const totalIgpuEnergy = updatedRecords.reduce((sum, r) => sum + r.igpu_energy, 0)
      const totalEnergy =
        totalCpuEnergy + totalGpuEnergy + totalMemEnergy + totalIgpuEnergy
      const energyKwh = totalEnergy / 1000

      // Calculate averages incrementally would be better, but let's at least avoid multiple maps
      let sumCpu = 0,
        sumGpu = 0,
        sumMem = 0
      for (const r of updatedRecords) {
        sumCpu += r.cpu_usage
        sumGpu += r.gpu_usage
        sumMem += r.mem_usage
      }

      return {
        ...prevRun,
        records: updatedRecords,
        totalEnergy,
        carbonFootprint: energyKwh * 0.475 * 1000,
        avgCpuUsage: sumCpu / count,
        avgGpuUsage: sumGpu / count,
        avgMemUsage: sumMem / count,
      }
    })
  }, [latestRecord])

  useEffect(() => {
    const fetchData = async () => {
      try {
        // Fetch all runs using the API client
        const runsData = await fetchAllRuns()
        setRuns(runsData)

        // Fetch the latest run with details
        if (runsData.length > 0) {
          const latestRunId = runsData[runsData.length - 1].id
          const runData = await fetchRunDetail(latestRunId)
          setCurrentRun(runData)
        }
      } catch (error) {
        console.error('Failed to fetch data:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchData()
  }, [])

  // Calculate aggregate stats
  const totalEnergy = runs.reduce((sum, r) => sum + r.totalEnergy, 0)
  const totalCoreHours = runs.reduce((sum, r) => sum + r.totalCoreHours, 0)
  const avgCpuUsage =
    runs.length > 0
      ? runs.reduce((sum, r) => sum + r.avgCpuUsage, 0) / runs.length
      : 0
  const totalCarbonFootprint = runs.reduce((sum, r) => sum + r.carbonFootprint, 0)
  const avgMemUsage =
    runs.length > 0
      ? runs.reduce((sum, r) => sum + r.avgMemUsage || 0, 0) / runs.length
      : 0

  // Calculate trends (comparing last two runs)
  const trend =
    runs.length >= 2
      ? {
        energyTrend:
          ((runs[runs.length - 2].totalEnergy -
            runs[runs.length - 1].totalEnergy) /
            runs[runs.length - 2].totalEnergy) *
          100,
        carbonTrend:
          ((runs[runs.length - 2].carbonFootprint -
            runs[runs.length - 1].carbonFootprint) /
            runs[runs.length - 2].carbonFootprint) *
          100,
      }
      : null

  return (
    <DashboardLayout>
      <div className="p-4 md:p-6 lg:p-8 w-full">
        {/* Header */}
        <div className="mb-6 md:mb-8">
          <h1 className="text-2xl md:text-3xl lg:text-4xl font-bold text-foreground mb-2">
            Dashboard
          </h1>
          <p className="text-sm md:text-base text-muted-foreground">
            Monitor your EcoCode runs and environmental impact
          </p>
        </div>

        {/* Project Overview */}
        <div className="mb-8">
          <h2 className="text-lg md:text-xl font-semibold text-foreground mb-4">
            Project Overview
          </h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 md:gap-4">
            <MetricCard
              icon={Zap}
              label="Total Energy"
              value={Math.round(totalEnergy).toLocaleString()}
              unit="Wh"
              trend={
                trend
                  ? {
                    value: trend.energyTrend,
                    direction: trend.energyTrend > 0 ? 'up' : 'down',
                  }
                  : undefined
              }
              loading={loading}
            />
            <MetricCard
              icon={Clock}
              label="Total Core Hours"
              value={(totalCoreHours / 24).toFixed(2)}
              unit="hrs"
              loading={loading}
            />
            <MetricCard
              icon={Leaf}
              label="Carbon Footprint"
              value={Math.round(totalCarbonFootprint).toLocaleString()}
              unit="g CO₂"
              trend={
                trend
                  ? {
                    value: trend.carbonTrend,
                    direction: trend.carbonTrend > 0 ? 'up' : 'down',
                  }
                  : undefined
              }
              loading={loading}
            />
            <MetricCard
              icon={Activity}
              label="Avg CPU Usage"
              value={avgCpuUsage.toFixed(1)}
              unit="%"
              loading={loading}
            />
            <MetricCard
              icon={Activity}
              label="Avg GPU Usage"
              value={(runs.reduce((sum, r) => sum + r.avgGpuUsage, 0) / (runs.length || 1)).toFixed(1)}
              unit="%"
              loading={loading}
            />
            <MetricCard
              icon={Activity}
              label="Avg Memory Usage"
              value={avgMemUsage.toFixed(1)}
              unit="%"
              loading={loading}
            />
          </div>
        </div>

        {/* Charts */}
        {currentRun && (
          <div className="mb-8 space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <EnergyChart data={currentRun.records} />
              <UtilizationChart data={currentRun.records} />
            </div>
          </div>
        )}

        {/* Recent Runs Table */}
        <RecentRunsTable runs={runs} />
      </div>
    </DashboardLayout>
  )
}
