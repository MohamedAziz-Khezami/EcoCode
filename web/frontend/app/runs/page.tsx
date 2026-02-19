'use client'

import { useEffect, useState } from 'react'
import { Search } from 'lucide-react'
import { DashboardLayout } from '@/components/DashboardLayout'
import { RunsTable } from '@/components/RunsTable'
import { Run } from '@/lib/mock-data'
import { fetchAllRuns } from '@/lib/api-client'

export default function RunsPage() {
  const [runs, setRuns] = useState<Omit<Run, 'records'>[]>([])
  const [filteredRuns, setFilteredRuns] = useState<Omit<Run, 'records'>[]>([])
  const [loading, setLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState('')

  useEffect(() => {
    const fetchRuns = async () => {
      try {
        const data = await fetchAllRuns()
        setRuns(data)
        setFilteredRuns(data)
      } catch (error) {
        console.error('Failed to fetch runs:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchRuns()
  }, [])

  const handleSearch = (query: string) => {
    setSearchQuery(query)

    if (!query.trim()) {
      setFilteredRuns(runs)
      return
    }

    const lowerQuery = query.toLowerCase()
    const filtered = runs.filter((run) => {
      // Search by run name
      if (run.name.toLowerCase().includes(lowerQuery)) return true

      // Parse filters like "status:finished" or "energy < 50"
      if (query.includes(':')) {
        const [key, value] = query.split(':')
        if (key.trim() === 'status' && value) {
          return run.status === value.trim()
        }
      }

      // Numeric comparison for energy
      if (query.includes('<') || query.includes('>')) {
        const match = query.match(/energy\s*([<>])\s*(\d+)/)
        if (match) {
          const [, operator, value] = match
          const threshold = parseFloat(value)
          return operator === '<'
            ? run.totalEnergy < threshold
            : run.totalEnergy > threshold
        }
      }

      return false
    })

    setFilteredRuns(filtered)
  }

  return (
    <DashboardLayout>
      <div className="p-4 md:p-6 lg:p-8 w-full">
        {/* Header */}
        <div className="mb-6 md:mb-8">
          <h1 className="text-2xl md:text-3xl lg:text-4xl font-bold text-foreground mb-2">
            Run History
          </h1>
          <p className="text-sm md:text-base text-muted-foreground">
            View and manage all your EcoCode runs
          </p>
        </div>

        {/* Search Bar */}
        <div className="mb-6">
          <div className="relative">
            <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-muted-foreground" />
            <input
              type="text"
              placeholder="Search runs... (e.g., 'Training Run', 'status:finished', 'energy < 100')"
              value={searchQuery}
              onChange={(e) => handleSearch(e.target.value)}
              className="w-full pl-12 pr-4 py-3 glass rounded-lg text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-0 transition-all duration-200"
            />
          </div>
        </div>

        {/* Results info */}
        <div className="mb-4 text-sm text-muted-foreground">
          Showing {filteredRuns.length} of {runs.length} runs
        </div>

        {/* Runs Table */}
        {filteredRuns.length > 0 ? (
          <RunsTable runs={filteredRuns} />
        ) : (
          <div className="chart-container text-center py-12">
            <p className="text-muted-foreground">
              {searchQuery
                ? 'No runs match your search criteria'
                : 'No runs found'}
            </p>
          </div>
        )}
      </div>
    </DashboardLayout>
  )
}
