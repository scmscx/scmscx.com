#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "postgres" <<-'EOSQL'
	CREATE USER "bounding.net" LOGIN password 'anotverysecurepassword';
	CREATE DATABASE "bounding.net";
	GRANT ALL PRIVILEGES ON DATABASE "bounding.net" TO "postgres";
	GRANT ALL PRIVILEGES ON DATABASE "bounding.net" TO "bounding.net";

	GRANT ALL PRIVILEGES ON DATABASE "postgres" TO "bounding.net";
EOSQL

psql -v ON_ERROR_STOP=1 --username "postgres" -d "bounding.net" <<-'EOSQL'
    ALTER SCHEMA public OWNER TO "bounding.net";


    --
    -- Name: pg_trgm; Type: EXTENSION; Schema: -; Owner: -
    --

    CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;

    --
    -- Name: EXTENSION pg_trgm; Type: COMMENT; Schema: -; Owner: 
    --

    COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';


    --
    -- Name: vector; Type: EXTENSION; Schema: -; Owner: -
    --

    CREATE EXTENSION IF NOT EXISTS vector WITH SCHEMA public;


    --
    -- Name: EXTENSION vector; Type: COMMENT; Schema: -; Owner: 
    --

    COMMENT ON EXTENSION vector IS 'vector data type and ivfflat and hnsw access methods';

    --
    -- PostgreSQL database dump
    --

    -- Dumped from database version 15.6 (Debian 15.6-1.pgdg110+2)
    -- Dumped by pg_dump version 16.6

    SET statement_timeout = 0;
    SET lock_timeout = 0;
    SET idle_in_transaction_session_timeout = 0;
    SET client_encoding = 'UTF8';
    SET standard_conforming_strings = on;
    SELECT pg_catalog.set_config('search_path', '', false);
    SET check_function_bodies = false;
    SET xmloption = content;
    SET client_min_messages = warning;
    SET row_security = off;


    --
    -- Name: decode_url_part(character varying); Type: FUNCTION; Schema: public; Owner: postgres
    --

    CREATE FUNCTION public.decode_url_part(p character varying) RETURNS character varying
        LANGUAGE sql IMMUTABLE STRICT
        AS $_$
    SELECT convert_from(CAST(E'\\x' || string_agg(CASE WHEN length(r.m[1]) = 1 THEN encode(convert_to(r.m[1], 'SQL_ASCII'), 'hex') ELSE substring(r.m[1] from 2 for 2) END, '') AS bytea), 'UTF8')
    FROM regexp_matches($1, '%[0-9a-f][0-9a-f]|.', 'gi') AS r(m);
    $_$;


    ALTER FUNCTION public.decode_url_part(p character varying) OWNER TO postgres;

    --
    -- Name: insert_string(bigint, text, text[]); Type: PROCEDURE; Schema: public; Owner: bounding.net
    --

    CREATE PROCEDURE public.insert_string(IN bigint, IN text, IN arr text[])
        LANGUAGE plpgsql
        AS $_$
    DECLARE
    str text;
    id int4;
    BEGIN
        foreach str in array $3
        loop
            insert into string (data) values (str) on conflict do nothing;
            commit;
            select string.id from string where data = str into id;
        
            insert into stringmap (map, string) values ($1, id) on conflict do nothing;
            commit;
        
            IF $2 = 'scenario_name' THEN
                update stringmap set scenario_name = true where map = $1 and string = id;
            ELSIF $2 = 'scenario_description' then
                update stringmap set scenario_description = true where map = $1 and string = id;
            ELSIF $2 = 'unit_names' then
                update stringmap set unit_names = true where map = $1 and string = id;
            ELSIF $2 = 'force_names' then
                update stringmap set force_names = true where map = $1 and string = id;
            end if;
            commit;
        end loop;
    END;
    $_$;


    ALTER PROCEDURE public.insert_string(IN bigint, IN text, IN arr text[]) OWNER TO "bounding.net";

    --
    -- Name: insert_string2(bigint, text, text[]); Type: PROCEDURE; Schema: public; Owner: bounding.net
    --

    CREATE PROCEDURE public.insert_string2(IN bigint, IN text, IN arr text[])
        LANGUAGE plpgsql
        AS $_$
    DECLARE
    str text;
    id int4;
    BEGIN
        foreach str in array $3
        loop
            IF $2 = 'scenario_name' then
                insert into stringmap2 (map, data, scenario_name) values ($1, str, true) on conflict (map, data) do update set scenario_name = true;
            ELSIF $2 = 'scenario_description' then
                insert into stringmap2 (map, data, scenario_description) values ($1, str, true) on conflict (map, data) do update set scenario_description = true;
            ELSIF $2 = 'unit_names' then
                insert into stringmap2 (map, data, unit_names) values ($1, str, true) on conflict (map, data) do update set unit_names = true;
            ELSIF $2 = 'force_names' then
                insert into stringmap2 (map, data, force_names) values ($1, str, true) on conflict (map, data) do update set force_names = true;
            ELSIF $2 = 'file_names' then
                insert into stringmap2 (map, data, file_names) values ($1, str, true) on conflict (map, data) do update set file_names = true;
            end if;
        end loop;
    END;
    $_$;


    ALTER PROCEDURE public.insert_string2(IN bigint, IN text, IN arr text[]) OWNER TO "bounding.net";

    --
    -- Name: insert_string3(bigint, text, text[]); Type: PROCEDURE; Schema: public; Owner: postgres
    --

    CREATE PROCEDURE public.insert_string3(IN bigint, IN text, IN arr text[])
        LANGUAGE plpgsql
        AS $_$
    DECLARE
    str text;
    id int4;
    BEGIN
        foreach str in array $3
        loop
            IF $2 = 'scenario_name' then
                insert into stringmap3 (map, data, scenario_name) values ($1, str, true);
            ELSIF $2 = 'scenario_description' then
                insert into stringmap3 (map, data, scenario_description) values ($1, str, true);
            ELSIF $2 = 'unit_names' then
                insert into stringmap3 (map, data, unit_names) values ($1, str, true);
            ELSIF $2 = 'force_names' then
                insert into stringmap3 (map, data, force_names) values ($1, str, true);
            ELSIF $2 = 'file_names' then
                insert into stringmap3 (map, data, file_names) values ($1, str, true);
            end if;
        end loop;
    END;
    $_$;


    ALTER PROCEDURE public.insert_string3(IN bigint, IN text, IN arr text[]) OWNER TO postgres;

    --
    -- Name: ngrams_array(text, integer); Type: FUNCTION; Schema: public; Owner: postgres
    --

    CREATE FUNCTION public.ngrams_array(word text, n integer) RETURNS text[]
        LANGUAGE plpgsql IMMUTABLE STRICT
        AS $$
            DECLARE
                    result text[];
            BEGIN
                    FOR i IN 1 .. length(word) - n + 1 LOOP
                            result := result || substr(lower(word), i, n);
                    END LOOP;

                    RETURN result;
            END;
    $$;


    ALTER FUNCTION public.ngrams_array(word text, n integer) OWNER TO postgres;

    --
    -- Name: ngrams_tsvector(tsvector, integer); Type: FUNCTION; Schema: public; Owner: postgres
    --

    CREATE FUNCTION public.ngrams_tsvector(vec tsvector, n integer) RETURNS tsvector
        LANGUAGE plpgsql IMMUTABLE STRICT
        AS $$
            declare
                    REC RECORD;
                    result text[];
            BEGIN
                for REC IN SELECT word FROM ts_stat('select vec') LOOP
                    FOR i IN 1 .. length(REC.word) - n + 1 LOOP
                            result := result || substr(lower(REC.word), i, n);
                    END LOOP;
                end loop;
                
                    RETURN result;
            END;
    $$;


    ALTER FUNCTION public.ngrams_tsvector(vec tsvector, n integer) OWNER TO postgres;

    SET default_tablespace = '';

    SET default_table_access_method = heap;

    --
    -- Name: account; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.account (
        id bigint NOT NULL,
        username text,
        passwordhash text,
        salt text,
        token text,
        isfake bigint DEFAULT '0'::bigint,
        created bigint DEFAULT (date_part('epoch'::text, now()))::bigint NOT NULL,
        default_playlist bigint
    );


    ALTER TABLE public.account OWNER TO "bounding.net";

    --
    -- Name: account_id_seq; Type: SEQUENCE; Schema: public; Owner: bounding.net
    --

    CREATE SEQUENCE public.account_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.account_id_seq OWNER TO "bounding.net";

    --
    -- Name: account_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: bounding.net
    --

    ALTER SEQUENCE public.account_id_seq OWNED BY public.account.id;


    --
    -- Name: cache; Type: TABLE; Schema: public; Owner: postgres
    --

    CREATE TABLE public.cache (
        data bytea,
        max_map_id bigint,
        original_size bigint
    );


    ALTER TABLE public.cache OWNER TO postgres;

    --
    -- Name: chkblob; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.chkblob (
        hash text NOT NULL,
        length bigint NOT NULL,
        ver bigint NOT NULL,
        data bytea NOT NULL
    );


    ALTER TABLE public.chkblob OWNER TO "bounding.net";

    --
    -- Name: chkdenorm; Type: TABLE; Schema: public; Owner: postgres
    --

    CREATE TABLE public.chkdenorm (
        width bigint,
        height bigint,
        tileset bigint,
        human_players bigint,
        computer_players bigint,
        sprites bigint,
        triggers bigint,
        briefing_triggers bigint,
        locations bigint,
        units bigint,
        scenario_name text,
        get_deaths_euds_or_epds bigint,
        set_deaths_euds_or_epds bigint,
        eups bigint,
        strings bigint,
        chkblob text NOT NULL,
        doodads bigint,
        scenario_description text
    );


    ALTER TABLE public.chkdenorm OWNER TO postgres;

    --
    -- Name: featuredmaps; Type: TABLE; Schema: public; Owner: postgres
    --

    CREATE TABLE public.featuredmaps (
        map_id bigint NOT NULL,
        rank bigint NOT NULL
    );


    ALTER TABLE public.featuredmaps OWNER TO postgres;

    --
    -- Name: filename; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.filename (
        id bigint NOT NULL,
        filename text
    );


    ALTER TABLE public.filename OWNER TO "bounding.net";

    --
    -- Name: filename_id_seq; Type: SEQUENCE; Schema: public; Owner: bounding.net
    --

    CREATE SEQUENCE public.filename_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.filename_id_seq OWNER TO "bounding.net";

    --
    -- Name: filename_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: bounding.net
    --

    ALTER SEQUENCE public.filename_id_seq OWNED BY public.filename.id;


    --
    -- Name: filenames2; Type: TABLE; Schema: public; Owner: postgres
    --

    CREATE TABLE public.filenames2 (
        map_id bigint NOT NULL,
        modified_time timestamp with time zone,
        filename_id bigint NOT NULL
    );


    ALTER TABLE public.filenames2 OWNER TO postgres;

    --
    -- Name: filetime; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.filetime (
        id bigint NOT NULL,
        map bigint NOT NULL,
        accessed_time bigint,
        modified_time bigint,
        creation_time bigint
    );


    ALTER TABLE public.filetime OWNER TO "bounding.net";

    --
    -- Name: filetime_id_seq; Type: SEQUENCE; Schema: public; Owner: bounding.net
    --

    CREATE SEQUENCE public.filetime_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.filetime_id_seq OWNER TO "bounding.net";

    --
    -- Name: filetime_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: bounding.net
    --

    ALTER SEQUENCE public.filetime_id_seq OWNED BY public.filetime.id;


    --
    -- Name: map; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.map (
        id bigint NOT NULL,
        uploaded_by bigint,
        uploaded_time bigint,
        chkblob text,
        views bigint DEFAULT 0,
        downloads bigint DEFAULT 0,
        last_viewed bigint,
        last_downloaded bigint,
        mapblob2 text NOT NULL,
        nsfw boolean DEFAULT false NOT NULL,
        outdated boolean DEFAULT false NOT NULL,
        broken boolean DEFAULT false NOT NULL,
        unfinished boolean DEFAULT false NOT NULL,
        denorm_scenario text,
        mapblob_size bigint NOT NULL,
        blackholed boolean DEFAULT false NOT NULL,
        spoiler_unit_names bool DEFAULT false NOT NULL
    );


    ALTER TABLE public.map OWNER TO "bounding.net";

    --
    -- Name: map_id_seq; Type: SEQUENCE; Schema: public; Owner: bounding.net
    --

    CREATE SEQUENCE public.map_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.map_id_seq OWNER TO "bounding.net";

    --
    -- Name: map_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: bounding.net
    --

    ALTER SEQUENCE public.map_id_seq OWNED BY public.map.id;


    --
    -- Name: mapfilename; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.mapfilename (
        id bigint NOT NULL,
        map bigint,
        filename bigint
    );


    ALTER TABLE public.mapfilename OWNER TO "bounding.net";

    --
    -- Name: mapfilename_id_seq; Type: SEQUENCE; Schema: public; Owner: bounding.net
    --

    CREATE SEQUENCE public.mapfilename_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.mapfilename_id_seq OWNER TO "bounding.net";

    --
    -- Name: mapfilename_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: bounding.net
    --

    ALTER SEQUENCE public.mapfilename_id_seq OWNED BY public.mapfilename.id;


    --
    -- Name: minimap; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.minimap (
        chkhash text NOT NULL,
        width integer,
        height integer,
        minimap bytea,
        ph16x16 bytea,
        ph32x32 bytea,
        ph8x8 bytea,
        vector bit(256) NOT NULL
    );


    ALTER TABLE public.minimap OWNER TO "bounding.net";

    --
    -- Name: playlist; Type: TABLE; Schema: public; Owner: postgres
    --

    CREATE TABLE public.playlist (
        id bigint NOT NULL,
        owner bigint NOT NULL,
        name text NOT NULL,
        time_created bigint DEFAULT (date_part('epoch'::text, now()))::bigint NOT NULL
    );


    ALTER TABLE public.playlist OWNER TO postgres;

    --
    -- Name: playlist_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
    --

    CREATE SEQUENCE public.playlist_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.playlist_id_seq OWNER TO postgres;

    --
    -- Name: playlist_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
    --

    ALTER SEQUENCE public.playlist_id_seq OWNED BY public.playlist.id;


    --
    -- Name: playlistmap; Type: TABLE; Schema: public; Owner: postgres
    --

    CREATE TABLE public.playlistmap (
        id bigint NOT NULL,
        playlist bigint NOT NULL,
        map bigint NOT NULL,
        time_created bigint DEFAULT (date_part('epoch'::text, now()))::bigint NOT NULL,
        prev bigint
    );


    ALTER TABLE public.playlistmap OWNER TO postgres;

    --
    -- Name: playlistmap_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
    --

    CREATE SEQUENCE public.playlistmap_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.playlistmap_id_seq OWNER TO postgres;

    --
    -- Name: playlistmap_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
    --

    ALTER SEQUENCE public.playlistmap_id_seq OWNED BY public.playlistmap.id;


    --
    -- Name: replay; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.replay (
        id bigint NOT NULL,
        hash text NOT NULL,
        uploaded_by bigint,
        uploaded_time bigint,
        denorm_game_creator bytea,
        denorm_time_saved bigint,
        denorm_frames bigint,
        denorm_number_of_human_players bigint,
        denorm_first_human_player bytea,
        denorm_scenario bytea,
        denorm_game bytea,
        chkhash text
    );


    ALTER TABLE public.replay OWNER TO "bounding.net";

    --
    -- Name: replay_id_seq; Type: SEQUENCE; Schema: public; Owner: bounding.net
    --

    CREATE SEQUENCE public.replay_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.replay_id_seq OWNER TO "bounding.net";

    --
    -- Name: replay_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: bounding.net
    --

    ALTER SEQUENCE public.replay_id_seq OWNED BY public.replay.id;


    --
    -- Name: replayblob; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.replayblob (
        hash text NOT NULL,
        data bytea NOT NULL
    );


    ALTER TABLE public.replayblob OWNER TO "bounding.net";

    --
    -- Name: string_id_seq; Type: SEQUENCE; Schema: public; Owner: bounding.net
    --

    CREATE SEQUENCE public.string_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.string_id_seq OWNER TO "bounding.net";

    --
    -- Name: stringmap2; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.stringmap2 (
        map bigint NOT NULL,
        data text NOT NULL,
        scenario_name boolean DEFAULT false NOT NULL,
        scenario_description boolean DEFAULT false NOT NULL,
        unit_names boolean DEFAULT false NOT NULL,
        force_names boolean DEFAULT false NOT NULL,
        file_names boolean DEFAULT false NOT NULL
    );


    ALTER TABLE public.stringmap2 OWNER TO "bounding.net";

    --
    -- Name: tag; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.tag (
        id bigint NOT NULL,
        key text,
        value text
    );


    ALTER TABLE public.tag OWNER TO "bounding.net";

    --
    -- Name: tag_id_seq; Type: SEQUENCE; Schema: public; Owner: bounding.net
    --

    CREATE SEQUENCE public.tag_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.tag_id_seq OWNER TO "bounding.net";

    --
    -- Name: tag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: bounding.net
    --

    ALTER SEQUENCE public.tag_id_seq OWNED BY public.tag.id;


    --
    -- Name: tagmap; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.tagmap (
        id bigint NOT NULL,
        map bigint,
        tag bigint
    );


    ALTER TABLE public.tagmap OWNER TO "bounding.net";

    --
    -- Name: tagmap_id_seq; Type: SEQUENCE; Schema: public; Owner: bounding.net
    --

    CREATE SEQUENCE public.tagmap_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.tagmap_id_seq OWNER TO "bounding.net";

    --
    -- Name: tagmap_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: bounding.net
    --

    ALTER SEQUENCE public.tagmap_id_seq OWNED BY public.tagmap.id;


    --
    -- Name: userlogs; Type: TABLE; Schema: public; Owner: bounding.net
    --

    CREATE TABLE public.userlogs (
        id bigint NOT NULL,
        log_time bigint NOT NULL,
        user_id bigint,
        ip_addr character varying,
        activity_type character varying,
        map_id bigint,
        replay_id bigint,
        chk_hash character varying,
        mapblob_hash character varying,
        username character varying,
        tac character varying,
        event character varying,
        method character varying,
        path character varying,
        query character varying,
        referer character varying,
        accept_language character varying,
        accept_encoding character varying,
        user_agent character varying,
        user_username character varying,
        user_token character varying,
        remote_addr character varying,
        trace_id character varying,
        host character varying,
        tracking_analytics_was_provided_by_request boolean,
        query_string character varying,
        version character varying,
        request_time_us bigint,
        error character varying,
        if_modified_since character varying,
        if_none_match character varying,
        sec_ch_ua_platform character varying,
        sec_ch_ua_mobile character varying,
        accept character varying,
        cookies character varying,
        status smallint,
        sec_ch_ua character varying
    );


    ALTER TABLE public.userlogs OWNER TO "bounding.net";

    --
    -- Name: user_stats; Type: MATERIALIZED VIEW; Schema: public; Owner: bounding.net
    --

    CREATE MATERIALIZED VIEW public.user_stats AS
    SELECT (count(*) / 30) AS views,
        30 AS days
    FROM public.userlogs
    WHERE (((now() - '30 days'::interval) < to_timestamp(((userlogs.log_time / 1000))::double precision)) AND starts_with((userlogs.path)::text, '/map/'::text) AND ((userlogs.user_agent)::text !~~* '%bot%'::text) AND ((userlogs.user_agent)::text !~~* '%bing%'::text) AND ((userlogs.user_agent)::text !~~* '%spider%'::text) AND ((userlogs.user_agent)::text !~~* '%http%'::text))
    UNION ALL
    SELECT (count(*) / 7) AS views,
        7 AS days
    FROM public.userlogs
    WHERE (((now() - '7 days'::interval) < to_timestamp(((userlogs.log_time / 1000))::double precision)) AND starts_with((userlogs.path)::text, '/map/'::text) AND ((userlogs.user_agent)::text !~~* '%bot%'::text) AND ((userlogs.user_agent)::text !~~* '%bing%'::text) AND ((userlogs.user_agent)::text !~~* '%spider%'::text) AND ((userlogs.user_agent)::text !~~* '%http%'::text))
    UNION ALL
    SELECT (count(*) / 1) AS views,
        1 AS days
    FROM public.userlogs
    WHERE (((now() - '1 day'::interval) < to_timestamp(((userlogs.log_time / 1000))::double precision)) AND starts_with((userlogs.path)::text, '/map/'::text) AND ((userlogs.user_agent)::text !~~* '%bot%'::text) AND ((userlogs.user_agent)::text !~~* '%bing%'::text) AND ((userlogs.user_agent)::text !~~* '%spider%'::text) AND ((userlogs.user_agent)::text !~~* '%http%'::text))
    WITH NO DATA;


    ALTER MATERIALIZED VIEW public.user_stats OWNER TO "bounding.net";

    --
    -- Name: userlogging_id_seq; Type: SEQUENCE; Schema: public; Owner: bounding.net
    --

    CREATE SEQUENCE public.userlogging_id_seq
        START WITH 1
        INCREMENT BY 1
        NO MINVALUE
        NO MAXVALUE
        CACHE 1;


    ALTER SEQUENCE public.userlogging_id_seq OWNER TO "bounding.net";

    --
    -- Name: userlogging_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: bounding.net
    --

    ALTER SEQUENCE public.userlogging_id_seq OWNED BY public.userlogs.id;


    --
    -- Name: account id; Type: DEFAULT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.account ALTER COLUMN id SET DEFAULT nextval('public.account_id_seq'::regclass);


    --
    -- Name: filename id; Type: DEFAULT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.filename ALTER COLUMN id SET DEFAULT nextval('public.filename_id_seq'::regclass);


    --
    -- Name: filetime id; Type: DEFAULT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.filetime ALTER COLUMN id SET DEFAULT nextval('public.filetime_id_seq'::regclass);


    --
    -- Name: map id; Type: DEFAULT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.map ALTER COLUMN id SET DEFAULT nextval('public.map_id_seq'::regclass);


    --
    -- Name: mapfilename id; Type: DEFAULT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.mapfilename ALTER COLUMN id SET DEFAULT nextval('public.mapfilename_id_seq'::regclass);


    --
    -- Name: playlist id; Type: DEFAULT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.playlist ALTER COLUMN id SET DEFAULT nextval('public.playlist_id_seq'::regclass);


    --
    -- Name: playlistmap id; Type: DEFAULT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.playlistmap ALTER COLUMN id SET DEFAULT nextval('public.playlistmap_id_seq'::regclass);


    --
    -- Name: replay id; Type: DEFAULT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.replay ALTER COLUMN id SET DEFAULT nextval('public.replay_id_seq'::regclass);


    --
    -- Name: tag id; Type: DEFAULT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.tag ALTER COLUMN id SET DEFAULT nextval('public.tag_id_seq'::regclass);


    --
    -- Name: tagmap id; Type: DEFAULT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.tagmap ALTER COLUMN id SET DEFAULT nextval('public.tagmap_id_seq'::regclass);


    --
    -- Name: userlogs id; Type: DEFAULT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.userlogs ALTER COLUMN id SET DEFAULT nextval('public.userlogging_id_seq'::regclass);


    --
    -- Name: account account_un; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.account
        ADD CONSTRAINT account_un UNIQUE (id);


    --
    -- Name: account account_un_username; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.account
        ADD CONSTRAINT account_un_username UNIQUE (username);


    --
    -- Name: chkblob chkblob_pkey; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.chkblob
        ADD CONSTRAINT chkblob_pkey PRIMARY KEY (hash);


    --
    -- Name: chkdenorm chkdenorm_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.chkdenorm
        ADD CONSTRAINT chkdenorm_pk PRIMARY KEY (chkblob);


    --
    -- Name: featuredmaps featuredmaps_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.featuredmaps
        ADD CONSTRAINT featuredmaps_pk PRIMARY KEY (map_id);


    --
    -- Name: filename filename_un; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.filename
        ADD CONSTRAINT filename_un UNIQUE (id);


    --
    -- Name: filename filename_un_filename; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.filename
        ADD CONSTRAINT filename_un_filename UNIQUE (filename);


    --
    -- Name: filenames2 filenames2_unique_1; Type: CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.filenames2
        ADD CONSTRAINT filenames2_unique_1 UNIQUE (map_id, modified_time, filename_id);


    --
    -- Name: filetime filetime_pkey; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.filetime
        ADD CONSTRAINT filetime_pkey PRIMARY KEY (id);


    --
    -- Name: filetime filetime_un; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.filetime
        ADD CONSTRAINT filetime_un UNIQUE (accessed_time, modified_time, creation_time, map);


    --
    -- Name: filename idx_53870_sqlite_autoindex_filename_1; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.filename
        ADD CONSTRAINT idx_53870_sqlite_autoindex_filename_1 PRIMARY KEY (id);


    --
    -- Name: tag idx_53879_sqlite_autoindex_tag_1; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.tag
        ADD CONSTRAINT idx_53879_sqlite_autoindex_tag_1 PRIMARY KEY (id);


    --
    -- Name: account idx_53894_sqlite_autoindex_account_1; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.account
        ADD CONSTRAINT idx_53894_sqlite_autoindex_account_1 PRIMARY KEY (id);


    --
    -- Name: tagmap idx_53904_sqlite_autoindex_tagmap_1; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.tagmap
        ADD CONSTRAINT idx_53904_sqlite_autoindex_tagmap_1 PRIMARY KEY (id);


    --
    -- Name: mapfilename idx_53910_sqlite_autoindex_mapfilename_1; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.mapfilename
        ADD CONSTRAINT idx_53910_sqlite_autoindex_mapfilename_1 PRIMARY KEY (id);


    --
    -- Name: map idx_53931_sqlite_autoindex_map_1; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.map
        ADD CONSTRAINT idx_53931_sqlite_autoindex_map_1 PRIMARY KEY (id);


    --
    -- Name: replay idx_53940_sqlite_autoindex_replay_1; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.replay
        ADD CONSTRAINT idx_53940_sqlite_autoindex_replay_1 PRIMARY KEY (id);


    --
    -- Name: map mapblob2_unique; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.map
        ADD CONSTRAINT mapblob2_unique UNIQUE (mapblob2);


    --
    -- Name: mapfilename mapfilename_un; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.mapfilename
        ADD CONSTRAINT mapfilename_un UNIQUE (id);


    --
    -- Name: mapfilename mapfilename_unique; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.mapfilename
        ADD CONSTRAINT mapfilename_unique UNIQUE (map, filename);


    --
    -- Name: minimap minimap_unique; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.minimap
        ADD CONSTRAINT minimap_unique UNIQUE (chkhash);


    --
    -- Name: stringmap2 newtable_pk; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.stringmap2
        ADD CONSTRAINT newtable_pk PRIMARY KEY (map, data);


    --
    -- Name: playlist playlist_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.playlist
        ADD CONSTRAINT playlist_pkey PRIMARY KEY (id);


    --
    -- Name: playlistmap playlistmap_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.playlistmap
        ADD CONSTRAINT playlistmap_pkey PRIMARY KEY (id);


    --
    -- Name: replay replay_un; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.replay
        ADD CONSTRAINT replay_un UNIQUE (id);


    --
    -- Name: replay replay_un_hash; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.replay
        ADD CONSTRAINT replay_un_hash UNIQUE (hash);


    --
    -- Name: replayblob replayblob_pkey; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.replayblob
        ADD CONSTRAINT replayblob_pkey PRIMARY KEY (hash);


    --
    -- Name: tag tag_un; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.tag
        ADD CONSTRAINT tag_un UNIQUE (id);


    --
    -- Name: tagmap tagmap_un; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.tagmap
        ADD CONSTRAINT tagmap_un UNIQUE (id);


    --
    -- Name: userlogs userlogging_pkey; Type: CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.userlogs
        ADD CONSTRAINT userlogging_pkey PRIMARY KEY (id);


    --
    -- Name: chkdenorm_briefing_triggers_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_briefing_triggers_idx ON public.chkdenorm USING btree (briefing_triggers);


    --
    -- Name: chkdenorm_computer_players_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_computer_players_idx ON public.chkdenorm USING btree (computer_players);


    --
    -- Name: chkdenorm_doodads_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_doodads_idx ON public.chkdenorm USING btree (doodads);


    --
    -- Name: chkdenorm_eups_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_eups_idx ON public.chkdenorm USING btree (eups);


    --
    -- Name: chkdenorm_get_deaths_euds_or_epds_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_get_deaths_euds_or_epds_idx ON public.chkdenorm USING btree (get_deaths_euds_or_epds);


    --
    -- Name: chkdenorm_height_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_height_idx ON public.chkdenorm USING btree (height);


    --
    -- Name: chkdenorm_human_players_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_human_players_idx ON public.chkdenorm USING btree (human_players);


    --
    -- Name: chkdenorm_locations_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_locations_idx ON public.chkdenorm USING btree (locations);


    --
    -- Name: chkdenorm_scenario_description_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_scenario_description_idx ON public.chkdenorm USING btree (scenario_description);


    --
    -- Name: chkdenorm_scenario_name_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_scenario_name_idx ON public.chkdenorm USING btree (scenario_name);


    --
    -- Name: chkdenorm_set_deaths_euds_or_epds_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_set_deaths_euds_or_epds_idx ON public.chkdenorm USING btree (set_deaths_euds_or_epds);


    --
    -- Name: chkdenorm_sprites_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_sprites_idx ON public.chkdenorm USING btree (sprites);


    --
    -- Name: chkdenorm_strings_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_strings_idx ON public.chkdenorm USING btree (strings);


    --
    -- Name: chkdenorm_tileset_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_tileset_idx ON public.chkdenorm USING btree (tileset);


    --
    -- Name: chkdenorm_triggers_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_triggers_idx ON public.chkdenorm USING btree (triggers);


    --
    -- Name: chkdenorm_units_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_units_idx ON public.chkdenorm USING btree (units);


    --
    -- Name: chkdenorm_width_idx; Type: INDEX; Schema: public; Owner: postgres
    --

    CREATE INDEX chkdenorm_width_idx ON public.chkdenorm USING btree (width);


    --
    -- Name: filename_filename_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX filename_filename_idx ON public.filename USING btree (filename);


    --
    -- Name: filetime_map_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX filetime_map_idx ON public.filetime USING btree (map);


    --
    -- Name: map_chkblob_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX map_chkblob_idx ON public.map USING btree (chkblob);


    --
    -- Name: map_downloads_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX map_downloads_idx ON public.map USING btree (downloads);


    --
    -- Name: map_last_downloaded_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX map_last_downloaded_idx ON public.map USING btree (last_downloaded);


    --
    -- Name: map_last_viewed_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX map_last_viewed_idx ON public.map USING btree (last_viewed);


    --
    -- Name: map_uploaded_by_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX map_uploaded_by_idx ON public.map USING btree (uploaded_by);


    --
    -- Name: map_uploaded_time_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX map_uploaded_time_idx ON public.map USING btree (uploaded_time);


    --
    -- Name: map_views_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX map_views_idx ON public.map USING btree (views);


    --
    -- Name: mapfilename_idx_filename; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX mapfilename_idx_filename ON public.mapfilename USING btree (filename);


    --
    -- Name: mapfilename_idx_map; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX mapfilename_idx_map ON public.mapfilename USING btree (map);


    --
    -- Name: minimap_vector_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX minimap_vector_idx ON public.minimap USING hnsw (vector public.bit_hamming_ops);


    --
    -- Name: replay_chkhash_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX replay_chkhash_idx ON public.replay USING btree (chkhash);


    --
    -- Name: replay_uploaded_time_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX replay_uploaded_time_idx ON public.replay USING btree (uploaded_time);


    --
    -- Name: tag_idx_key; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX tag_idx_key ON public.tag USING btree (key);


    --
    -- Name: tagmap_idx_map; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX tagmap_idx_map ON public.tagmap USING btree (map);


    --
    -- Name: tagmap_idx_tag; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX tagmap_idx_tag ON public.tagmap USING btree (tag);


    --
    -- Name: trgm_idx_stringmap2_gin; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX trgm_idx_stringmap2_gin ON public.stringmap2 USING gin (data public.gin_trgm_ops);


    --
    -- Name: trgm_idx_tag_value; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX trgm_idx_tag_value ON public.tag USING gin (value public.gin_trgm_ops);


    --
    -- Name: user_stats_days_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE UNIQUE INDEX user_stats_days_idx ON public.user_stats USING btree (days);


    --
    -- Name: userlogs_log_time_idx; Type: INDEX; Schema: public; Owner: bounding.net
    --

    CREATE INDEX userlogs_log_time_idx ON public.userlogs USING btree (to_timestamp(((log_time / 1000))::double precision) DESC);


    --
    -- Name: account account_fk; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.account
        ADD CONSTRAINT account_fk FOREIGN KEY (default_playlist) REFERENCES public.playlist(id);


    --
    -- Name: chkdenorm chkdenorm_chkblob_fk; Type: FK CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.chkdenorm
        ADD CONSTRAINT chkdenorm_chkblob_fk FOREIGN KEY (chkblob) REFERENCES public.chkblob(hash) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: featuredmaps featuredmaps_map_fk; Type: FK CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.featuredmaps
        ADD CONSTRAINT featuredmaps_map_fk FOREIGN KEY (map_id) REFERENCES public.map(id) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: filenames2 filenames2_filename_fk; Type: FK CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.filenames2
        ADD CONSTRAINT filenames2_filename_fk FOREIGN KEY (filename_id) REFERENCES public.filename(id) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: filenames2 filenames2_map_fk; Type: FK CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.filenames2
        ADD CONSTRAINT filenames2_map_fk FOREIGN KEY (map_id) REFERENCES public.map(id) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: filetime filetime_fk; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.filetime
        ADD CONSTRAINT filetime_fk FOREIGN KEY (map) REFERENCES public.map(id) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: map map_chkblob_fk; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.map
        ADD CONSTRAINT map_chkblob_fk FOREIGN KEY (chkblob) REFERENCES public.chkblob(hash) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: map map_uploaded_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.map
        ADD CONSTRAINT map_uploaded_by_fkey FOREIGN KEY (uploaded_by) REFERENCES public.account(id);


    --
    -- Name: mapfilename mapfilename_filename_fkey; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.mapfilename
        ADD CONSTRAINT mapfilename_filename_fkey FOREIGN KEY (filename) REFERENCES public.filename(id) ON UPDATE RESTRICT ON DELETE RESTRICT;


    --
    -- Name: mapfilename mapfilename_map_fkey; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.mapfilename
        ADD CONSTRAINT mapfilename_map_fkey FOREIGN KEY (map) REFERENCES public.map(id) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: minimap minimap_chkhash_fkey; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.minimap
        ADD CONSTRAINT minimap_chkhash_fkey FOREIGN KEY (chkhash) REFERENCES public.chkblob(hash) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: playlistmap playlist_fk; Type: FK CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.playlistmap
        ADD CONSTRAINT playlist_fk FOREIGN KEY (playlist) REFERENCES public.playlist(id) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: playlistmap prev_fk; Type: FK CONSTRAINT; Schema: public; Owner: postgres
    --

    ALTER TABLE ONLY public.playlistmap
        ADD CONSTRAINT prev_fk FOREIGN KEY (prev) REFERENCES public.playlistmap(id) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: replay replay_fk; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.replay
        ADD CONSTRAINT replay_fk FOREIGN KEY (hash) REFERENCES public.replayblob(hash);


    --
    -- Name: replay replay_fk_chkhash; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.replay
        ADD CONSTRAINT replay_fk_chkhash FOREIGN KEY (chkhash) REFERENCES public.chkblob(hash);


    --
    -- Name: stringmap2 stringmap2_map_fk; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.stringmap2
        ADD CONSTRAINT stringmap2_map_fk FOREIGN KEY (map) REFERENCES public.map(id) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: tagmap tagmap_map_fkey; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.tagmap
        ADD CONSTRAINT tagmap_map_fkey FOREIGN KEY (map) REFERENCES public.map(id) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: tagmap tagmap_tag_fkey; Type: FK CONSTRAINT; Schema: public; Owner: bounding.net
    --

    ALTER TABLE ONLY public.tagmap
        ADD CONSTRAINT tagmap_tag_fkey FOREIGN KEY (tag) REFERENCES public.tag(id) ON UPDATE RESTRICT ON DELETE CASCADE;


    --
    -- Name: SCHEMA public; Type: ACL; Schema: -; Owner: postgres
    --

    REVOKE USAGE ON SCHEMA public FROM PUBLIC;
    GRANT ALL ON SCHEMA public TO PUBLIC;


    --
    -- Name: TABLE cache; Type: ACL; Schema: public; Owner: postgres
    --

    GRANT ALL ON TABLE public.cache TO "bounding.net";


    --
    -- Name: TABLE chkdenorm; Type: ACL; Schema: public; Owner: postgres
    --

    GRANT SELECT,INSERT,DELETE ON TABLE public.chkdenorm TO "bounding.net";


    --
    -- Name: TABLE featuredmaps; Type: ACL; Schema: public; Owner: postgres
    --

    GRANT SELECT ON TABLE public.featuredmaps TO "bounding.net";


    --
    -- Name: TABLE filenames2; Type: ACL; Schema: public; Owner: postgres
    --

    GRANT ALL ON TABLE public.filenames2 TO "bounding.net";


    --
    -- Name: TABLE playlist; Type: ACL; Schema: public; Owner: postgres
    --

    GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.playlist TO "bounding.net";


    --
    -- Name: SEQUENCE playlist_id_seq; Type: ACL; Schema: public; Owner: postgres
    --

    GRANT ALL ON SEQUENCE public.playlist_id_seq TO "bounding.net";


    --
    -- Name: TABLE playlistmap; Type: ACL; Schema: public; Owner: postgres
    --

    GRANT SELECT,INSERT,DELETE,UPDATE ON TABLE public.playlistmap TO "bounding.net";


    --
    -- Name: SEQUENCE playlistmap_id_seq; Type: ACL; Schema: public; Owner: postgres
    --

    GRANT ALL ON SEQUENCE public.playlistmap_id_seq TO "bounding.net";


    --
    -- PostgreSQL database dump complete
    --
EOSQL

psql -v ON_ERROR_STOP=1 --username "postgres" -d "bounding.net" <<-'EOSQL'
    insert into account (id, username, passwordhash, salt, token, isfake, created, default_playlist) values (10, 'anonymous', null, null, null, 1, extract(epoch from now()), null);
EOSQL
