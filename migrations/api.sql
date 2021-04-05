CREATE TABLE api.profiles (
  id SERIAL PRIMARY KEY,
  slug varchar(64) NOT NULL UNIQUE,
  user_id integer REFERENCES authentication.users (id) ON UPDATE CASCADE ON DELETE CASCADE, 
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- projects 
CREATE TABLE api.projects (
  id SERIAL PRIMARY KEY,
  slug varchar(64) NOT NULL UNIQUE,
  is_public boolean NOT NULL DEFAULT FALSE,
  name varchar(64),
  description text,
  url text,
  owner_id integer REFERENCES authentication.users (id) ON UPDATE CASCADE ON DELETE CASCADE, 
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE api.projects ENABLE ROW LEVEL SECURITY;

-- repositories 
CREATE TABLE api.repositories (
  id SERIAL PRIMARY KEY,
  name varchar(64) NOT NULL,
  is_public boolean NOT NULL,
  project_id integer REFERENCES api.projects (id) ON UPDATE CASCADE ON DELETE CASCADE, 
  slug varchar(64) NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE api.repositories ENABLE ROW LEVEL SECURITY;

CREATE TABLE api.memberships (
  id SERIAL PRIMARY KEY,
  project_id integer REFERENCES api.projects (id) ON UPDATE CASCADE ON DELETE CASCADE, 
  user_id integer REFERENCES authentication.users (id) ON UPDATE CASCADE ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE api.memberships ENABLE ROW LEVEL SECURITY;


CREATE OR REPLACE FUNCTION api.create_project (slug text, is_public boolean DEFAULT FALSE, name text DEFAULT NULL, description text DEFAULT NULL)
  RETURNS integer
  AS $$
DECLARE
  _project_id integer;
  _user_id integer;
BEGIN
  SELECT api.current_user_id() INTO _user_id;
  INSERT INTO api.projects (slug, owner_id) values(slug, _user_id) RETURNING id INTO _project_id;
  RETURN _project_id;
END;
$$
LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION api.add_member (project_id integer, member_email text)
  RETURNS integer
  AS $$
DECLARE
  _project_id integer;
  _user_id integer;
BEGIN
  SELECT api.current_user_id() INTO _user_id;
  INSERT INTO api.projects (slug, owner_id) values(slug, _user_id) RETURNING id INTO _project_id;
  RETURN _project_id;
END;
$$
LANGUAGE plpgsql
SECURITY DEFINER;

CREATE OR REPLACE FUNCTION api.create_token ()
  RETURNS integer
  AS $$
DECLARE
  _project_id integer;
  _user_id integer;
BEGIN
  SELECT api.current_user_id() INTO _user_id;
  INSERT INTO api.projects (slug, owner_id) values(slug, _user_id) RETURNING id INTO _project_id;
  RETURN _project_id;
END;
$$
LANGUAGE plpgsql
SECURITY DEFINER;

