'use client'

import { useEffect, useState, useMemo } from 'react'
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
import { ProjectsTable } from '@/components/ProjectsTable'
import { Run } from '@/lib/mock-data'
import { fetchAllRuns, fetchRunDetail, fetchAllProjects, ProjectSummary } from '@/lib/api-client'
import { useSSE } from '@/hooks/use-sse'

export default function Dashboard() {
  const [runs, setRuns] = useState<Omit<Run, 'records'>[]>([])
  const [projects, setProjects] = useState<ProjectSummary[]>([])
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

      // Calculate new total and averages incrementally (O(1) instead of O(N))
      const newTotalEnergy =
        prevRun.totalEnergy +
        (latestRecord.cpu_energy +
          latestRecord.gpu_energy +
          latestRecord.mem_energy +
          latestRecord.igpu_energy)

      const energyKwh = newTotalEnergy / 1000

      // Incremental averages: (old_avg * old_count + new_val) / new_count
      const avgCpuUsage = (prevRun.avgCpuUsage * (count - 1) + latestRecord.cpu_usage) / count
      const avgGpuUsage = (prevRun.avgGpuUsage * (count - 1) + latestRecord.gpu_usage) / count
      const avgMemUsage = (prevRun.avgMemUsage * (count - 1) + latestRecord.mem_usage) / count

      return {
        ...prevRun,
        records: updatedRecords,
        totalEnergy: newTotalEnergy,
        carbonFootprint: energyKwh * 0.475 * 1000,
        avgCpuUsage,
        avgGpuUsage,
        avgMemUsage,
      }
    })
  }, [latestRecord])

  useEffect(() => {
    const fetchData = async () => {
      try {
        const projData = await fetchAllProjects()
        setProjects(projData)

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

  // Calculate aggregate stats (memoized to avoid re-calculating on every real-time update)
  const stats = useMemo(() => {
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
    const avgGpuUsage =
      runs.length > 0
        ? runs.reduce((sum, r) => sum + r.avgGpuUsage, 0) / runs.length
        : 0

    return {
      totalEnergy,
      totalCoreHours,
      avgCpuUsage,
      totalCarbonFootprint,
      avgMemUsage,
      avgGpuUsage
    }
  }, [runs])

  // Calculate trends (comparing last two runs)
  const trend = useMemo(() => {
    if (runs.length < 2) return null

    const prevRun = runs[runs.length - 2]
    const lastRun = runs[runs.length - 1]

    return {
      energyTrend:
        ((prevRun.totalEnergy - lastRun.totalEnergy) /
          prevRun.totalEnergy) *
        100,
      carbonTrend:
        ((prevRun.carbonFootprint - lastRun.carbonFootprint) /
          prevRun.carbonFootprint) *
        100,
    }
  }, [runs])

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
              value={Math.round(stats.totalEnergy).toLocaleString()}
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
              value={(stats.totalCoreHours / 24).toFixed(2)}
              unit="hrs"
              loading={loading}
            />
            {/* <MetricCard
              icon={Leaf}
              label="Carbon Footprint"
              value={Math.round(stats.totalCarbonFootprint).toLocaleString()}
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
            /> */}
            <MetricCard
              icon={Activity}
              label="Avg CPU Usage"
              value={stats.avgCpuUsage.toFixed(1)}
              unit="%"
              loading={loading}
            />
            <MetricCard
              icon={Activity}
              label="Avg GPU Usage"
              value={stats.avgGpuUsage.toFixed(1)}
              unit="%"
              loading={loading}
            />
            <MetricCard
              icon={Activity}
              label="Avg Memory Usage"
              value={stats.avgMemUsage.toFixed(1)}
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

        {/* Projects Table */}
        <ProjectsTable projects={projects} />
      </div>
    </DashboardLayout>
  )
}
