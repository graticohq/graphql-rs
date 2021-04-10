-- First we’ll need a table to keep track of our users:

CREATE TABLE IF NOT EXISTS authentication.users (
  id SERIAL PRIMARY KEY,
  email text NOT NULL UNIQUE CHECK (email ~* '^.+@.+\..+$'),
  password text NOT NULL CHECK (length(password) < 512),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Next we’ll use the pgcrypto extension and a trigger to keep passwords safe in the users table.

CREATE OR REPLACE FUNCTION authentication.encrypt_pass ()
  RETURNS TRIGGER
  AS $$
BEGIN
  IF tg_op = 'INSERT' OR new.password <> old.password THEN
    new.password = crypt(new.password, gen_salt('bf'));
  END IF;
  RETURN new;
END
$$
LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS encrypt_pass ON authentication.users;

CREATE TRIGGER encrypt_pass
  BEFORE INSERT
  OR UPDATE ON authentication.users FOR EACH ROW
  EXECUTE PROCEDURE authentication.encrypt_pass ();

CREATE OR REPLACE FUNCTION api.current_user_id()
  RETURNS integer
  AS $$
DECLARE
  _user_id integer;
BEGIN
  SELECT
    u.id
  FROM
    authentication.tokens t
    INNER JOIN authentication.users u ON u.id = t.user_id
  WHERE
    t.token = current_setting('session.token', TRUE)
  LIMIT 1 INTO _user_id;
  IF _user_id IS NULL THEN
    raise invalid_password
    USING message = 'invalid user or password';
  END IF;
  RETURN _user_id;
END;
$$
LANGUAGE plpgsql
SECURITY DEFINER;


-- With the table in place we can make a helper to check a password against the encrypted column. It returns the user_id for a user if the email and password are correct.

CREATE OR REPLACE FUNCTION authentication.check_password (email text, password text)
  RETURNS integer
  LANGUAGE plpgsql
  AS $$
BEGIN
  RETURN (
    SELECT
      id
    FROM
      authentication.users
    WHERE
      users.email = user_role.email
      AND users.password = crypt(user_role.password, users.password));
END;
$$;

CREATE TYPE api.token AS (token text, exp integer);

CREATE OR REPLACE FUNCTION api.login (email text, password text)
  RETURNS api.token
  AS $$
DECLARE
  _user_id integer;
  _token text;
  _token_id integer;
BEGIN
  SELECT
    authentication.check_password (email,
      password) INTO _user_id;
  IF _user_id IS NULL THEN
    raise invalid_password
    USING message = 'invalid user or password';
  END IF;

  INSERT INTO authentication.tokens (user_id) values(_user_id) RETURNING id, token INTO _token_id, _token;
  -- add a admin scope(3) for user profile/projects/repos
  INSERT INTO authentication.scopes (token_id, profile_clearance, projects_clearance, repositories_clearance) values (_token_id, 3, 3, 3);
  RETURN (_token, extract(epoch FROM now())::integer + 60 * 60 * 24 * 180)::api.token;
END;
$$
LANGUAGE plpgsql
SECURITY DEFINER;


CREATE OR REPLACE FUNCTION api.register (slug text, email text, password text)
  RETURNS api.token
  AS $$
DECLARE
  _user_id integer;
BEGIN
  INSERT INTO authentication.users (email, password) values(email, password) RETURNING id INTO _user_id;
  INSERT INTO api.profiles (slug, user_id) values(slug, _user_id);
  RETURN api.login(email, password);
END;
$$
LANGUAGE plpgsql
SECURITY DEFINER;


