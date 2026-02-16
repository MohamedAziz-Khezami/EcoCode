'use client'

import Link from 'next/link'
import { Run } from '@/lib/mock-data'
import { format } from 'date-fns'
import { ChevronRight } from 'lucide-react'

interface RecentRunsTableProps {
  runs: Omit<Run, 'records'>[]
}

export function RecentRunsTable({ runs }: RecentRunsTableProps) {
  const recentRuns = runs.slice(0, 5)

  return (
    <div className="chart-container">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-foreground">Recent Runs</h3>
        <Link
          href="/runs"
          className="text-sm text-primary hover:text-primary/80 transition-colors"
        >
          View all →
        </Link>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-white/10">
              <th className="text-left py-3 px-4 text-muted-foreground font-medium">
                Run Name
              </th>
              <th className="text-right py-3 px-4 text-muted-foreground font-medium">
                Energy (Wh)
              </th>
              <th className="text-right py-3 px-4 text-muted-foreground font-medium">
                CO₂ (g)
              </th>
              <th className="text-center py-3 px-4 text-muted-foreground font-medium">
                Status
              </th>
              <th className="text-right py-3 px-4" />
            </tr>
          </thead>
          <tbody>
            {recentRuns.map((run) => (
              <tr
                key={run.id}
                className="border-b border-white/5 hover:bg-white/5 transition-colors"
              >
                <td className="py-3 px-4">
                  <div>
                    <p className="font-medium text-foreground">{run.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {format(new Date(run.timestamp), 'MMM d, HH:mm')}
                    </p>
                  </div>
                </td>
                <td className="text-right py-3 px-4 text-foreground font-medium">
                  {Math.round(run.totalEnergy).toLocaleString()}
                </td>
                <td className="text-right py-3 px-4 text-foreground font-medium">
                  {Math.round(run.carbonFootprint).toLocaleString()}
                </td>
                <td className="text-center py-3 px-4">
                  <span
                    className={`inline-block px-3 py-1 rounded-full text-xs font-medium ${
                      run.status === 'finished'
                        ? 'bg-emerald-500/20 text-emerald-300'
                        : run.status === 'running'
                          ? 'bg-blue-500/20 text-blue-300'
                          : 'bg-red-500/20 text-red-300'
                    }`}
                  >
                    {run.status}
                  </span>
                </td>
                <td className="text-right py-3 px-4">
                  <Link href={`/runs/${run.id}`}>
                    <ChevronRight className="w-4 h-4 text-muted-foreground hover:text-primary transition-colors" />
                  </Link>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
