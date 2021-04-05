CREATE TABLE authentication.scopes (
  id SERIAL PRIMARY KEY,
  token_id integer REFERENCES authentication.tokens (id) ON UPDATE CASCADE ON DELETE CASCADE, 
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  profile_clearance integer NOT NULL CHECK (profile_clearance < 4) DEFAULT 0,
  projects_clearance integer NOT NULL CHECK (projects_clearance < 4) DEFAULT 0,
  repositories_clearance integer NOT NULL CHECK (repositories_clearance < 4) DEFAULT 0,
  project_id integer REFERENCES api.projects (id) ON UPDATE CASCADE ON DELETE CASCADE, 
  repository_id integer REFERENCES api.repositories (id) ON UPDATE CASCADE ON DELETE CASCADE,
  project_clearance integer NOT NULL CHECK (projects_clearance < 4) DEFAULT 0,
  repositorie_clearance integer NOT NULL CHECK (repositories_clearance < 4) DEFAULT 0
);


