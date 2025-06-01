WITH cron_jobs AS (
	SELECT 
		jobid
		, jobname
		, schedule 
		, command 
	FROM cron.job 
	WHERE 
		active = TRUE 
	ORDER BY 1 ASC
)
, currently_running_jobs AS (
	SELECT 
		jobid AS job_id
		, status 
		, start_time 
	FROM cron.job_run_details 
	WHERE 
		status = 'running'
)
, job_runs AS ( 
	SELECT 
		j.jobid AS job_id 
		, j.jobname AS job_name 
		, d.status AS latest_completed_run_status 
		, d.return_message 
		, d.start_time 
		, d.end_time 
        , EXTRACT(EPOCH FROM (d.end_time - d.start_time)) AS run_duration_sec
		, AVG(EXTRACT(EPOCH FROM (d.end_time - d.start_time))) 
			OVER (PARTITION BY j.jobid ORDER BY d.start_time ASC ROWS BETWEEN 9 PRECEDING AND CURRENT ROW) AS rolling_last_10_runs_runtime_avg_sec
		, j.schedule 
		, j.command 
		, d.runid AS latest_run_id
		, d.job_pid AS latest_pid
		, ROW_NUMBER() OVER (PARTITION BY d.jobid ORDER BY d.start_time DESC) AS row_num 
	FROM cron_jobs AS j 
	LEFT JOIN cron.job_run_details AS d
		ON d.jobid = j.jobid 
		AND end_time IS NOT NULL
)
SELECT 
	jr.job_id 
	, jr.job_name 
	, CASE 
		WHEN crj.job_id IS NOT NULL 
		THEN TRUE 
		ELSE FALSE
		END AS is_currently_running
	, crj.start_time AS current_run_start_at
	, jr.rolling_last_10_runs_runtime_avg_sec
	, jr.rolling_last_10_runs_runtime_avg_sec/60 AS rolling_last_10_runs_runtime_avg_min
	, jr.run_duration_sec
	, jr.run_duration_sec/60 AS run_duration_min
	, jr.latest_completed_run_status 
	, jr.return_message
	, jr.start_time 
	, jr.end_time 
	, jr.schedule 
	, jr.command 
	, jr.latest_run_id 
	, jr.latest_pid
FROM job_runs AS jr 
LEFT JOIN currently_running_jobs AS crj 
	ON jr.job_id = crj.job_id 
WHERE 
	jr.row_num = 1 
ORDER BY jr.job_id ASC, jr.start_time DESC;