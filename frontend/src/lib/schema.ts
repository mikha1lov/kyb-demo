import { z } from 'zod'

export const entityTypes = ['sole_proprietor', 'llc', 'corporation', 'partnership'] as const
export type EntityType = (typeof entityTypes)[number]

export const entityTypeLabels: Record<EntityType, string> = {
  sole_proprietor: 'Sole proprietor',
  llc: 'LLC',
  corporation: 'Corporation',
  partnership: 'Partnership',
}

export const businessSchema = z.object({
  country: z.string().length(2, 'Select a country'),
  entityType: z.enum(entityTypes),
  legalName: z.string().trim().min(2, 'Enter the legal name'),
  registrationNumber: z.string().trim().min(5, 'Enter the registration number'),
  address: z.string().trim().min(5, 'Enter the registered address'),
})

export type Business = z.infer<typeof businessSchema>

export const companySchema = businessSchema.pick({ country: true, entityType: true })

export type Company = z.infer<typeof companySchema>

export const detailsSchema = businessSchema.pick({ legalName: true, registrationNumber: true, address: true })

export type Details = z.infer<typeof detailsSchema>

export const requirements = ['formation', 'source_of_funds', 'proof_of_address'] as const
export type Requirement = (typeof requirements)[number]

export const requirementLabels: Record<Requirement, string> = {
  formation: 'Registration certificate',
  source_of_funds: 'Source of funds',
  proof_of_address: 'Proof of address',
}
