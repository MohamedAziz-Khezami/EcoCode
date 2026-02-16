import { getRuns } from '@/lib/mock-data'

export async function GET() {
  try {
    const runs = getRuns()
    return Response.json(runs)
  } catch (error) {
    return Response.json(
      { error: 'Failed to fetch runs' },
      { status: 500 }
    )
  }
}
