
CREATE OR REPLACE FUNCTION authentication.check_project_access (projectid integer, clearance integer)
  RETURNS boolean
  AS $$
DECLARE
  _valid_token_id integer;
BEGIN
  RETURN 
    EXISTS (
      SELECT
        1
      FROM
        authentication.tokens t
        INNER JOIN authentication.scopes ps ON ps.token_id = t.id AND (ps.projects_clearance >= clearance OR (ps.project_id = projectid AND ps.project_clearance >= clearance))
        FULL JOIN api.memberships m ON m.user_id = t.user_id
        FULL JOIN api.projects p ON ( p.owner_id = t.user_id) OR (m.project_id = projectid
          AND m.user_id = t.user_id)
      WHERE
        t.token = current_setting('session.token', true) AND p.id = projectid);
END;
$$
LANGUAGE plpgsql
SECURITY DEFINER;

CREATE POLICY api_projects_policy ON api.projects
    USING (authentication.check_project_access(id, 1))
    WITH CHECK (authentication.check_project_access(id, 2));


GRANT USAGE ON SCHEMA api TO anonymous;
GRANT SELECT, UPDATE, INSERT ON ALL TABLES IN SCHEMA api TO anonymous;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA api TO anonymous;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA api TO anonymous;

GRANT anonymous TO authenticator;
GRANT anonymous TO api_user;
