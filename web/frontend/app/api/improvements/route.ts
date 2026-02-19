import { getImprovements } from '@/lib/mock-data'

export async function GET() {
  try {
    const improvements = getImprovements()
    return Response.json(improvements)
  } catch (error) {
    return Response.json(
      { error: 'Failed to fetch improvements' },
      { status: 500 }
    )
  }
}
