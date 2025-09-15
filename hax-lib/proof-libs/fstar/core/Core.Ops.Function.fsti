module Core.Ops.Function

[@FStar.Tactics.Typeclasses.tcclass]
class t_FnOnce
  (v_Self: Type0) (v_Args: Type0)
  = {
  f_Output:Type0;
  f_call_once_pre:v_Self -> v_Args
    -> Type0;
  f_call_once_post:
      v_Self ->
      v_Args ->
      f_Output
    -> Type0;
  f_call_once:x0: v_Self -> x1: v_Args
    -> Prims.Pure f_Output
        (f_call_once_pre x0 x1)
        (fun result ->
            f_call_once_post x0 x1 result)
}

[@FStar.Tactics.Typeclasses.tcclass]
class t_Fn (v_Self: Type0) (v_Args: Type0)
  = {
  [@@@ FStar.Tactics.Typeclasses.no_method]_super_11844238443650317928:t_FnOnce
    v_Self v_Args;
  f_call_pre:v_Self -> v_Args -> Type0;
  f_call_post:v_Self -> v_Args -> _
    -> Type0;
  f_call:x0: v_Self -> x1: v_Args
    -> Prims.Pure _super_11844238443650317928.f_Output
        (f_call_pre x0 x1)
        (fun result ->
            f_call_post x0 x1 result)
}
