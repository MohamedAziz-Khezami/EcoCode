'use client'

import Link from 'next/link'
import { useState } from 'react'
import { Run } from '@/lib/mock-data'
import { format } from 'date-fns'
import { ChevronRight, Check, Clock, X } from 'lucide-react'

interface RunsTableProps {
  runs: Omit<Run, 'records'>[]
}

export function RunsTable({ runs }: RunsTableProps) {
  const [selectedRuns, setSelectedRuns] = useState<Set<string>>(new Set())

  const toggleRunSelection = (runId: string) => {
    const newSelected = new Set(selectedRuns)
    if (newSelected.has(runId)) {
      newSelected.delete(runId)
    } else {
      newSelected.add(runId)
    }
    setSelectedRuns(newSelected)
  }

  const toggleAllRuns = () => {
    if (selectedRuns.size === runs.length) {
      setSelectedRuns(new Set())
    } else {
      setSelectedRuns(new Set(runs.map((r) => r.id)))
    }
  }

  const getStatusIcon = (
    status: 'running' | 'finished' | 'failed'
  ) => {
    switch (status) {
      case 'finished':
        return <Check className="w-4 h-4 text-emerald-400" />
      case 'running':
        return <Clock className="w-4 h-4 text-blue-400 animate-spin" />
      case 'failed':
        return <X className="w-4 h-4 text-red-400" />
    }
  }

  return (
    <div className="chart-container">
      {/* Controls */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={selectedRuns.size === runs.length && runs.length > 0}
              onChange={toggleAllRuns}
              className="w-4 h-4 rounded border-border bg-secondary cursor-pointer"
            />
            <span className="text-sm text-muted-foreground">
              {selectedRuns.size > 0
                ? `${selectedRuns.size} selected`
                : 'Select all'}
            </span>
          </label>
        </div>
      </div>

      {/* Table */}
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-white/10">
              <th className="text-left py-4 px-4 w-10">
                <input
                  type="checkbox"
                  checked={
                    selectedRuns.size === runs.length && runs.length > 0
                  }
                  onChange={toggleAllRuns}
                  className="w-4 h-4 rounded border-border bg-secondary cursor-pointer"
                />
              </th>
              <th className="text-left py-4 px-4 text-muted-foreground font-medium">
                Run Name
              </th>
              <th className="text-center py-4 px-4 text-muted-foreground font-medium">
                Status
              </th>
              <th className="text-right py-4 px-4 text-muted-foreground font-medium">
                Duration
              </th>
              <th className="text-right py-4 px-4 text-muted-foreground font-medium">
                Energy (Wh)
              </th>
              <th className="text-right py-4 px-4 text-muted-foreground font-medium">
                Avg CPU
              </th>
              <th className="text-right py-4 px-4 text-muted-foreground font-medium">
                CO₂ (g)
              </th>
              <th className="text-right py-4 px-4" />
            </tr>
          </thead>
          <tbody>
            {runs.map((run) => {
              const isSelected = selectedRuns.has(run.id)
              return (
                <tr
                  key={run.id}
                  className={`border-b border-white/5 transition-colors ${
                    isSelected ? 'bg-primary/10' : 'hover:bg-white/5'
                  }`}
                >
                  <td className="py-4 px-4">
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => toggleRunSelection(run.id)}
                      className="w-4 h-4 rounded border-border bg-secondary cursor-pointer"
                    />
                  </td>
                  <td className="py-4 px-4">
                    <div>
                      <p className="font-medium text-foreground">{run.name}</p>
                      <p className="text-xs text-muted-foreground">
                        {format(new Date(run.timestamp), 'MMM d, yyyy HH:mm')}
                      </p>
                    </div>
                  </td>
                  <td className="py-4 px-4 text-center">
                    <div className="flex items-center justify-center">
                      {getStatusIcon(run.status)}
                    </div>
                  </td>
                  <td className="text-right py-4 px-4 text-foreground font-medium">
                    {Math.round(run.duration / 60)}m
                  </td>
                  <td className="text-right py-4 px-4 text-foreground font-medium">
                    {Math.round(run.totalEnergy).toLocaleString()}
                  </td>
                  <td className="text-right py-4 px-4 text-foreground font-medium">
                    {run.avgCpuUsage.toFixed(1)}%
                  </td>
                  <td className="text-right py-4 px-4 text-foreground font-medium">
                    {Math.round(run.carbonFootprint).toLocaleString()}
                  </td>
                  <td className="text-right py-4 px-4">
                    <Link href={`/runs/${run.id}`}>
                      <ChevronRight className="w-4 h-4 text-muted-foreground hover:text-primary transition-colors" />
                    </Link>
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </div>
  )
}
