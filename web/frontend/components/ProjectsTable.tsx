'use client'

import React, { memo } from 'react'
import Link from 'next/link'
import { ProjectSummary } from '@/lib/api-client'
import { format } from 'date-fns'
import { ChevronRight, Folder } from 'lucide-react'

interface ProjectsTableProps {
    projects: ProjectSummary[]
}

export const ProjectsTable = memo(function ProjectsTable({ projects }: ProjectsTableProps) {
    return (
        <div className="chart-container">
            <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-foreground flex items-center gap-2">
                    <Folder className="w-5 h-5 text-primary" />
                    Projects
                </h3>
                <Link
                    href="/projects"
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
                                Project Name
                            </th>
                            <th className="text-center py-3 px-4 text-muted-foreground font-medium">
                                Total Runs
                            </th>
                            <th className="text-right py-3 px-4 text-muted-foreground font-medium">
                                Total Energy (Wh)
                            </th>
                            <th className="text-right py-3 px-4 text-muted-foreground font-medium">
                                Last Run
                            </th>
                            <th className="text-right py-3 px-4" />
                        </tr>
                    </thead>
                    <tbody>
                        {projects.length === 0 ? (
                            <tr>
                                <td colSpan={5} className="py-8 text-center text-muted-foreground">
                                    No projects found.
                                </td>
                            </tr>
                        ) : projects.map((project) => (
                            <tr
                                key={project.id}
                                className="border-b border-white/5 hover:bg-white/5 transition-colors"
                            >
                                <td className="py-3 px-4">
                                    <p className="font-medium text-foreground">{project.name}</p>
                                </td>
                                <td className="text-center py-3 px-4 text-foreground font-medium">
                                    {project.totalRuns}
                                </td>
                                <td className="text-right py-3 px-4 text-foreground font-medium">
                                    {Math.round(project.totalEnergy).toLocaleString()}
                                </td>
                                <td className="text-right py-3 px-4 text-muted-foreground">
                                    {project.lastRunTimestamp ? format(new Date(project.lastRunTimestamp), 'MMM d, yyyy') : '-'}
                                </td>
                                <td className="text-right py-3 px-4">
                                    <Link href={`/projects/${project.id}`}>
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
})
