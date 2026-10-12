create table applications (
  id uuid primary key default gen_random_uuid(),
  status text not null default 'draft' check (status in ('draft', 'submitted')),
  business jsonb,
  sources jsonb not null default '{}'::jsonb,
  submitted_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table documents (
  id uuid primary key default gen_random_uuid(),
  application_id uuid not null references applications (id) on delete cascade,
  filename text not null,
  size_bytes integer not null,
  kind text not null,
  extracted jsonb not null,
  created_at timestamptz not null default now()
);

create index documents_application_id_idx on documents (application_id);
