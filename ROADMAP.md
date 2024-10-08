# Roadmap

## Cycle Roadmap
1. [ ] Proof-Of-Concept Cycle
	1. [X] Fetching archives
		1. [X] Fetcher exists and can fetch from a hardcoded store
		2. [X] DB exists and has basic config about a store
		3. [X] Fetcher can query the DB for store info
	2. [X] Licensing
		1. [X] We have a license that's as permissive as possible while preventing direct competition from using our code
	3. [X] Jobs
		1. [X] API layer and daemon now exist
		2. [X] API layer can create jobs
		3. [X] API layer can use temp storage
		4. [X] Daemon can execute jobs
	4. [ ] Nix Primitives
		1. [X] The CLI exists
		2. [X] The CLI can create/extract bare Nix archives (not closures)
		3. [ ] The CLI can calculate Nix closures
		4. [ ] The CLI can produce `.narinfo` files
	5. [ ] Pushing archives
		1. [X] "Push archive" is job type and functions correctly
2. [ ] Multi-tenancy
	1. [ ] Stores vs. Caches
		1. [X] Stores and caches are now separate things in the DB
		2. [X] The fetcher queries the DB to check if something is in a cache
		3. [X] The fetcher can fetch from multiple caches
		4. [X] Cache boundaries are enforced
	2. [ ] Keys Init
		1. [ ] Stopgap solution for a signing key exists
		2. [ ] Keys exist in the DB with permissions, an expiry date, and a revocation flag
		3. [X] Caches can be marked as private
		4. [X] Keys are required for anything but fetching from public caches
		5. [X] Keys are checked for required permissions by fetcher and API layers before performing actions
	3. [ ] Orgs and Users Init
		1. [X] Orgs and users exist in the database
		2. [ ] User credentials are kept and checked, and users can be logged in and given a "session token"
		3. [X] Keys are now tied to users
		4. [ ] CLI can log in as a user and get a session token
3. [ ] Authorization
	1. [ ] DB keeps track of all user permissions
	2. [ ] API layer actions are separated into session-authenticated actions and key-authenticated actions, where any key-authenticated action can also be executed with session authentication
	3. [ ] Super users and org owners are now separated from regular users
	4. [ ] Session-authenticated actions now include:
		1. [ ] Issuing and revoking keys
		2. [ ] Creating and deleting caches
		3. [ ] Super user actions:
			1. [ ] Creating and deleting stores
			2. [ ] Modifying permissions and user status for other users
	5. [ ] CLI can perform session-authenticated actions
4. [ ] Frontend
	1. [ ] Init
		1. [ ] The frontend exists
		2. [ ] Users can log in using the auth built in 2.3
		3. [ ] Users can view basic cache and store info
	2. [ ] Procedurally add all session and key-authenticated actions to frontend
	3. [ ] Add cache explorer
	4. [ ] Add public cache explorer for public caches
5. [ ] Billing
6. Unfinished
	1. After step 5, this is about a v0.1 release. Things that follow this are (in no particular order):
		- A hosted storage solution using R2
		- A migration tool for moving caches and stores
		- Free tier for open source
		- Garbage collection
		- Path deduplication from external public caches
		- Path deduplication between first-party caches
		- Chunked/alternative archive formats
		- Custom domains
