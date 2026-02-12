use crate::db::{with_connection, Database};
use crate::error::AppError;
use crate::focus::FocusManager;
use crate::models::FocusSchedule;
use crate::validation::{
    validate_budget_minutes, validate_budget_secs, validate_days_of_week, validate_time_format,
};
use std::sync::{Arc, Mutex};
use tauri::State;

use super::{CreateScheduleRequest, FocusScheduleResponse, FocusStateResponse, UpdateScheduleRequest};

#[tauri::command]
pub fn get_focus_state(focus_manager: State<Arc<FocusManager>>) -> Result<FocusStateResponse, String> {
    let state = focus_manager.get_state().map_err(|e| AppError::from(e).to_string())?;
    Ok(FocusStateResponse {
        active: state.active,
        budget_remaining: state.budget_remaining,
        session_duration_secs: state.session_duration_secs,
    })
}

#[tauri::command]
pub fn start_focus_session(
    focus_manager: State<Arc<FocusManager>>,
    budget_minutes: i32,
) -> Result<(), String> {
    let budget_secs = validate_budget_minutes(budget_minutes)?;
    focus_manager.start_session(budget_secs).map_err(|e| AppError::from(e).to_string())?;
    Ok(())
}

#[tauri::command]
pub fn end_focus_session(focus_manager: State<Arc<FocusManager>>) -> Result<(), String> {
    focus_manager.end_session().map_err(|e| AppError::from(e).to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_focus_schedules(db: State<Arc<Mutex<Database>>>) -> Result<Vec<FocusScheduleResponse>, String> {
    let schedules = with_connection(&db, |conn| FocusSchedule::find_all(conn))?;
    Ok(schedules.into_iter().map(FocusScheduleResponse::from).collect())
}

#[tauri::command]
pub fn create_focus_schedule(
    db: State<Arc<Mutex<Database>>>,
    request: CreateScheduleRequest,
) -> Result<FocusScheduleResponse, String> {
    validate_schedule_request(
        &request.start_time,
        &request.end_time,
        &request.days_of_week,
        request.distraction_budget_secs,
    )?;

    let db = db.lock().map_err(|_| AppError::LockPoisoned.to_string())?;
    let conn = db.connection();

    let mut schedule = FocusSchedule::new(
        &request.days_of_week,
        &request.start_time,
        &request.end_time,
        request.distraction_budget_secs,
    );
    schedule.save(conn).map_err(|e| AppError::from(e).to_string())?;

    Ok(FocusScheduleResponse::from(schedule))
}

#[tauri::command]
pub fn update_focus_schedule(
    db: State<Arc<Mutex<Database>>>,
    request: UpdateScheduleRequest,
) -> Result<FocusScheduleResponse, String> {
    validate_schedule_request(
        &request.start_time,
        &request.end_time,
        &request.days_of_week,
        request.distraction_budget_secs,
    )?;

    let db = db.lock().map_err(|_| AppError::LockPoisoned.to_string())?;
    let conn = db.connection();

    if FocusSchedule::find_by_id(conn, request.id)
        .map_err(|e| AppError::from(e).to_string())?
        .is_none()
    {
        return Err(AppError::NotFound { entity: "Schedule" }.to_string());
    }

    let schedule = FocusSchedule {
        id: Some(request.id),
        days_of_week: request.days_of_week,
        start_time: request.start_time,
        end_time: request.end_time,
        distraction_budget: request.distraction_budget_secs,
        enabled: request.enabled,
    };
    schedule.update(conn).map_err(|e| AppError::from(e).to_string())?;

    Ok(FocusScheduleResponse::from(schedule))
}

#[tauri::command]
pub fn delete_focus_schedule(db: State<Arc<Mutex<Database>>>, id: i64) -> Result<bool, String> {
    let result = with_connection(&db, |conn| FocusSchedule::delete(conn, id))?;
    Ok(result)
}

#[tauri::command]
pub fn get_active_schedule(
    focus_manager: State<Arc<FocusManager>>,
) -> Result<Option<FocusScheduleResponse>, String> {
    let schedule = focus_manager.get_active_schedule().map_err(|e| AppError::from(e).to_string())?;
    Ok(schedule.map(FocusScheduleResponse::from))
}

#[tauri::command]
pub fn check_focus_schedules(focus_manager: State<Arc<FocusManager>>) -> Result<(), String> {
    focus_manager.check_schedules().map_err(|e| AppError::from(e).to_string())?;
    Ok(())
}

fn validate_schedule_request(
    start_time: &str,
    end_time: &str,
    days_of_week: &str,
    budget_secs: i32,
) -> Result<(), String> {
    validate_time_format(start_time)?;
    validate_time_format(end_time)?;
    validate_days_of_week(days_of_week)?;
    validate_budget_secs(budget_secs)?;
    Ok(())
}
