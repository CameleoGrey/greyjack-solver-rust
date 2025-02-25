
import os
import sys
from pathlib import Path

# To launch normally from console
script_dir_path = Path(os.path.dirname(os.path.realpath(__file__)))
project_dir_id = script_dir_path.parts.index("python_client")
project_dir_path = Path(*script_dir_path.parts[:project_dir_id+1])
sys.path.append(str(project_dir_path))

import orjson
import pika
from persistence.DomainBuilder import DomainBuilder

def on_solution_receive(ch, method, properties, body):
    
    body = body.decode("utf-8")
    if body == "Solving finished":
        connection.close()
        print("done")
        try:
            sys.exit(0)
        except SystemExit:
            os._exit(0)

    domain_json = orjson.loads(body)
    domain = DomainBuilder().build_domain_from_json(domain_json)
    domain.print_metrics()
    #domain.print_paths()
    print()

if __name__ == "__main__":

    """
    Simplified client (without frontend, API facade, OAuth, etc, etc, etc) for GreyJack VRP solving service
    just to check, that observer mechanism (and the solver and solution built on top of it) works as I planned.
    """

    dir_path = os.path.dirname(os.path.realpath(__file__))
    data_dir_path = Path(project_dir_path, "..", "..", "data", "vrp", "data", "import")
    #file_path = Path(data_dir_path, "vrpweb", "basic", "air", "A-n32-k5.vrp")
    #file_path = Path(data_dir_path, "vrpweb", "basic", "air", "F-n135-k7.vrp")
    #file_path = Path(data_dir_path, "usa", "basic", "air", "usa-n100-k10.vrp")
    #file_path = Path(data_dir_path, "belgium", "basic", "air", "belgium-n50-k10.vrp")
    #file_path = Path(data_dir_path, "belgium", "basic", "air", "belgium-n500-k20.vrp")
    #file_path = Path(data_dir_path, "belgium", "basic", "air", "belgium-n1000-k40.vrp") #optimum: ~57.7; first_fit: ~195.3
    #file_path = Path(data_dir_path, "belgium", "basic", "air", "belgium-n2750-k55.vrp")
    #file_path = Path(data_dir_path, "belgium", "multidepot-timewindowed", "air", "belgium-tw-d2-n50-k10.vrp") # 16.005 - best possible
    file_path = Path(data_dir_path, "belgium", "multidepot-timewindowed", "air", "belgium-tw-d5-n500-k20.vrp")
    #file_path = Path(data_dir_path, "belgium", "multidepot-timewindowed", "air", "belgium-tw-d8-n1000-k40.vrp")

    task_data_dict = DomainBuilder()._read_vrp_file(file_path, for_json=True)
    task_data_dict["user_id"] = 13
    task_data_dict["task_id"] = 45

    task_data_json = orjson.dumps(task_data_dict)
    connection = pika.BlockingConnection(pika.ConnectionParameters(host="192.168.0.189", port=5672))
    task_data_sender = connection.channel()
    task_data_sender.queue_bind(queue="vrp_task_data", exchange="vrp_exchange")
    task_data_sender.basic_publish(exchange="vrp_exchange", routing_key="vrp_task_data", body=task_data_json)

    # TODO: create queue with solutions for specific user and task ids in the Rust part
    # TODO: bind client to the queue with solutions for specific user and task ids
    solution_receiver = connection.channel()
    solution_receiver.queue_bind(queue="vrp_solutions", exchange="vrp_solutions_exchange")
    solution_receiver.basic_consume(queue="vrp_solutions", on_message_callback=on_solution_receive, auto_ack=True)
    solution_receiver.start_consuming()

    connection.close()
    print("done")