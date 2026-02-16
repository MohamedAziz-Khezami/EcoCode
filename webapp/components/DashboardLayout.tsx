import { Sidebar } from './Sidebar'

export function DashboardLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <div className="flex flex-col lg:flex-row h-screen bg-background overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-auto">
        <div className="min-h-screen bg-gradient-to-br from-background via-background to-secondary/5">
          {children}
        </div>
      </main>
    </div>
  )
}
