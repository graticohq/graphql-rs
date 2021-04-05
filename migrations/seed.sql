select * from api.register('abhishiv', 'abhishiv@gmail.com', 'abhishiv');

select set_config('session.token', (select token from authentication.tokens order by id desc limit 1), true);
select * from api.create_project('gratico', true);

select * from api.register('abhishiv2', 'abhishiv2@gmail.com', 'abhishiv2');
select set_config('session.token', (select token from authentication.tokens order by id desc limit 1), true);
select * from api.create_project('gratico2', true);
