use std::time::Duration;
use snmp::SyncSession;

pub fn query() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let system_oid      = &[1,3,6,1,2,1,1,];
    let agent_addr      = "192.168.1.110:161";
    let community       = b"public";
    let timeout         = Duration::from_secs(2);
    let non_repeaters   = 0;
    let max_repetitions = 7; // number of items in "system" OID

    let mut sess = match SyncSession::new(agent_addr, community, Some(timeout), 0) {
        Ok(sess_out) => sess_out,
        Err(e) => return Err(format!("{e:?}").into()), 
    };

    let response = match sess.getbulk(&[system_oid], non_repeaters, max_repetitions) {
        Ok(res_out) => res_out,
        Err(e) => return Err(format!("{e:?}").into()), 
    };

    for (name, val) in response.varbinds {
        println!("{name} => {val:?}");
    }
    Ok(())
}
